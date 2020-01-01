use async_trait::async_trait;
use parking_lot::{RwLock, RwLockWriteGuard};

use std::collections::HashMap;
use std::io::{Read, Write};
use std::ops::FnOnce;

use crate::user::Username;

use super::data::Delta;
use super::data::*;
use super::line::LineIter;
use super::PaintResult;

#[async_trait]
pub trait AsyncProc<T> {
    async fn call(self, lock: &RwLock<BlockInfo>) -> PaintResult<T>;
}

struct ReadProc<F, T: 'static>(F)
where
    F: FnOnce(&BlockInfo) -> PaintResult<T> + Send;
struct WriteProc<F, T: 'static>(F)
where
    F: FnOnce(&mut BlockInfo) -> PaintResult<T> + Send;

#[async_trait]
impl<F, T: 'static> AsyncProc<T> for ReadProc<F, T>
where
    F: FnOnce(&BlockInfo) -> PaintResult<T> + Send,
{
    async fn call(self, lock: &RwLock<BlockInfo>) -> PaintResult<T> {
        let read = lock.read();
        self.0(&read)
    }
}

#[async_trait]
impl<F, T: 'static> AsyncProc<T> for WriteProc<F, T>
where
    F: FnOnce(&mut BlockInfo) -> PaintResult<T> + Send,
{
    async fn call(self, lock: &RwLock<BlockInfo>) -> PaintResult<T> {
        let mut write = lock.write();
        self.0(&mut write)
    }
}

pub struct PaintDB {
    blocks: RwLock<HashMap<BlockPos, RwLock<BlockInfo>>>,
}

impl PaintDB {
    async fn write_block<F, T: 'static>(&self, blk: BlockPos, proc: F) -> PaintResult<T>
    where
        F: FnOnce(&mut BlockInfo) -> PaintResult<T> + Send,
    {
        self.process_block(blk, WriteProc(proc)).await
    }

    async fn read_block<F, T: 'static>(&self, blk: BlockPos, proc: F) -> PaintResult<T>
    where
        F: FnOnce(&BlockInfo) -> PaintResult<T> + Send,
    {
        self.process_block(blk, ReadProc(proc)).await
    }

    async fn process_block<F, T: 'static>(&self, blk: BlockPos, proc: F) -> PaintResult<T>
    where
        F: AsyncProc<T>,
    {
        let read = self.blocks.read();
        if let Some(p) = read.get(&blk) {
            return proc.call(p).await;
        }
        drop(read);
        self.load_block(blk, proc).await
    }

    async fn load_block<F, T: 'static>(&self, blk: BlockPos, proc: F) -> PaintResult<T>
    where
        F: AsyncProc<T>,
    {
        let mut write = self.blocks.write();
        if write.get(&blk).is_none() {
            write.insert(blk, RwLock::new(BlockInfo::new()));
        }
        let read = RwLockWriteGuard::downgrade(write);
        match read.get(&blk) {
            Some(p) => proc.call(p).await,
            None => unreachable!(),
        }
    }
}

impl PaintDB {
    pub fn new() -> Self {
        Self {
            blocks: RwLock::new(HashMap::new()),
        }
    }
    pub async fn draw_pixels<I>(&self, user: &str, color: RGBA, pixels: I) -> PaintResult<usize>
    where
        I: IntoIterator<Item = PixelPos>,
    {
        let mut pixels = pixels.into_iter().peekable();
        let mut offsets = Vec::new();
        let mut success_cnt = 0;
        while let Some(p) = pixels.next() {
            let blk = p.block();
            offsets.push(p.offset());
            while let Some(p) = pixels.peek() {
                if p.block() != blk {
                    break;
                }
                offsets.push(p.offset());
                pixels.next();
            }
            let ok = self
                .write_block(blk, |info| {
                    Ok(info.draw_pixels(user, color, offsets.iter().cloned()))
                })
                .await?;
            if ok {
                success_cnt += offsets.len();
            }
            offsets.clear();
        }
        Ok(success_cnt)
    }

    pub async fn draw_lines<I>(
        &self,
        user: &str,
        color: RGBA,
        mut start: PixelPos,
        deltas: I,
    ) -> PaintResult<usize>
    where
        I: IntoIterator<Item = Delta>,
    {
        let mut success_cnt = 0;
        for d in deltas {
            success_cnt += self
                .draw_pixels(user, color, LineIter::new(start, d))
                .await?;
            start = start + d;
        }
        success_cnt += self
            .draw_pixels(user, color, LineIter::new(start, Delta { x: 0, y: 1 }))
            .await?;
        Ok(success_cnt)
    }

    pub async fn set_block<R: Read + Send>(
        &self,
        user: &str,
        blk: BlockPos,
        src: R,
    ) -> PaintResult<bool> {
        self.write_block(blk, |info| info.draw_block_from_png(user, src))
            .await
    }

    pub async fn get_block<W: Write + Send>(
        &self,
        blk: BlockPos,
        dst: W,
        ts: u64,
    ) -> PaintResult<u64> {
        self.read_block(blk, |info| info.block_to_png(dst, ts))
            .await
    }

    pub async fn set_lock(&self, user: Username, blk: BlockPos) -> PaintResult<bool> {
        self.write_block(blk, |info| Ok(info.set_owner(user))).await
    }

    pub async fn get_lock(&self, blk: BlockPos) -> PaintResult<Username> {
        self.read_block(blk, |info| Ok(info.get_owner())).await
    }

    pub async fn del_lock(&self, user: &str, blk: BlockPos) -> PaintResult<bool> {
        self.write_block(blk, |info| Ok(info.reset_owner(user)))
            .await
    }
}

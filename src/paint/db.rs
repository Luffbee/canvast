use parking_lot::{RwLock, RwLockWriteGuard};

use std::collections::HashMap;
use std::io::{Read, Write};

use crate::user::Username;

use super::data::Delta;
use super::data::*;
use super::line::LineIter;
use super::PaintResult;

pub trait PaintDB: Send + Sync {
    fn new() -> Self;
    fn draw_pixels<I>(&self, user: &str, color: RGBA, pixels: I) -> PaintResult<usize>
    where
        I: IntoIterator<Item = PixelPos>;
    fn draw_lines<I>(
        &self,
        user: &str,
        color: RGBA,
        start: PixelPos,
        deltas: I,
    ) -> PaintResult<usize>
    where
        I: IntoIterator<Item = Delta>;
    fn set_block<R: Read>(&self, user: &str, blk: BlockPos, src: R) -> PaintResult<bool>;
    fn get_block<W: Write>(&self, blk: BlockPos, dst: W, ts: u64) -> PaintResult<u64>;
    fn set_lock(&self, user: Username, blk: BlockPos) -> PaintResult<bool>;
    fn get_lock(&self, blk: BlockPos) -> PaintResult<Username>;
    fn del_lock(&self, user: &str, blk: BlockPos) -> PaintResult<bool>;
}

pub struct SharedDB {
    blocks: RwLock<HashMap<BlockPos, RwLock<BlockInfo>>>,
}

impl SharedDB {
    fn write_block<F, T>(&self, blk: BlockPos, proc: F) -> PaintResult<T>
    where
        F: FnOnce(&mut BlockInfo) -> PaintResult<T>,
    {
        self.process_block(blk, |lock| {
            let mut write = lock.write();
            proc(&mut write)
        })
    }

    fn read_block<F, T>(&self, blk: BlockPos, proc: F) -> PaintResult<T>
    where
        F: FnOnce(&BlockInfo) -> PaintResult<T>,
    {
        self.process_block(blk, |lock| {
            let read = lock.write();
            proc(&read)
        })
    }

    fn process_block<F, T>(&self, blk: BlockPos, proc: F) -> PaintResult<T>
    where
        F: FnOnce(&RwLock<BlockInfo>) -> PaintResult<T>,
    {
        let read = self.blocks.read();
        if let Some(p) = read.get(&blk) {
            return proc(p);
        }
        drop(read);
        self.load_block(blk, proc)
    }

    fn load_block<F, T>(&self, blk: BlockPos, proc: F) -> PaintResult<T>
    where
        F: FnOnce(&RwLock<BlockInfo>) -> PaintResult<T>,
    {
        let mut write = self.blocks.write();
        if write.get(&blk).is_none() {
            write.insert(blk, RwLock::new(BlockInfo::new()));
        }
        let read = RwLockWriteGuard::downgrade(write);
        match read.get(&blk) {
            Some(p) => proc(p),
            None => unreachable!(),
        }
    }
}

impl PaintDB for SharedDB {
    fn new() -> Self {
        SharedDB {
            blocks: RwLock::new(HashMap::new()),
        }
    }
    fn draw_pixels<I>(&self, user: &str, color: RGBA, pixels: I) -> PaintResult<usize>
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
            let ok = self.write_block(blk, |info| {
                Ok(info.draw_pixels(user, color, offsets.iter().cloned()))
            })?;
            if ok {
                success_cnt += offsets.len();
            }
            offsets.clear();
        }
        Ok(success_cnt)
    }

    fn draw_lines<I>(
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
            success_cnt += self.draw_pixels(user, color, LineIter::new(start, d))?;
            start = start + d;
        }
        success_cnt += self.draw_pixels(user, color, LineIter::new(start, Delta { x: 0, y: 1 }))?;
        Ok(success_cnt)
    }

    fn set_block<R: Read>(&self, user: &str, blk: BlockPos, src: R) -> PaintResult<bool> {
        self.write_block(blk, |info| info.draw_block_from_png(user, src))
    }

    fn get_block<W: Write>(&self, blk: BlockPos, dst: W, ts: u64) -> PaintResult<u64> {
        self.read_block(blk, |info| info.block_to_png(dst, ts))
    }

    fn set_lock(&self, user: Username, blk: BlockPos) -> PaintResult<bool> {
        self.write_block(blk, |info| Ok(info.set_owner(user)))
    }

    fn get_lock(&self, blk: BlockPos) -> PaintResult<Username> {
        self.read_block(blk, |info| Ok(info.get_owner()))
    }

    fn del_lock(&self, user: &str, blk: BlockPos) -> PaintResult<bool> {
        self.write_block(blk, |info| Ok(info.reset_owner(user)))
    }
}

use hex::FromHex;

use std::io::{Read, Write};
use std::ops::Add;

use crate::user::Username;

use super::error::{InternalError, PaintError, PaintResult};
use super::now;

#[derive(Clone, Copy, Eq, PartialEq, Deserialize, Serialize, Default)]
pub struct PixelPos {
    pub x: i64,
    pub y: i64,
}

impl PixelPos {
    pub fn block(&self) -> BlockPos {
        BlockPos {
            x: self.x >> BLOCK_BITS,
            y: self.y >> BLOCK_BITS,
        }
    }
    pub fn offset(&self) -> Offset {
        let x = ((self.x as usize) & (BLOCK_SIZE - 1)) as u8;
        let y = ((self.y as usize) & (BLOCK_SIZE - 1)) as u8;
        (x, y)
    }
}

impl Add<Delta> for PixelPos {
    type Output = PixelPos;
    fn add(self, rhs: Delta) -> PixelPos {
        PixelPos {
            x: self.x.wrapping_add(rhs.x as i64),
            y: self.y.wrapping_add(rhs.y as i64),
        }
    }
}

#[derive(Deserialize, Clone, Copy, Hash, Eq, PartialEq, Debug)]
pub struct BlockPos {
    pub x: i64,
    pub y: i64,
}

impl Add<Offset> for BlockPos {
    type Output = Self;
    fn add(self, rhs: Offset) -> Self {
        Self {
            x: self.x + rhs.0 as i64,
            y: self.y + rhs.1 as i64,
        }
    }
}

impl Into<PixelPos> for BlockPos {
    fn into(self) -> PixelPos {
        PixelPos {
            x: self.x << BLOCK_BITS,
            y: self.y << BLOCK_BITS,
        }
    }
}

pub type Offset = (u8, u8);

#[derive(Deserialize, Clone, Copy)]
pub struct Delta {
    pub x: i16,
    pub y: i16,
}

#[derive(Clone, Copy)]
pub struct RGB([u8; 3]);
#[derive(Clone, Copy)]
pub struct RGBA([u8; 4]);

impl FromHex for RGBA {
    type Error = PaintError;
    fn from_hex<T: AsRef<[u8]>>(hex: T) -> PaintResult<Self> {
        match <[u8; 4]>::from_hex(hex) {
            Ok(v) => Ok(RGBA(v)),
            Err(_) => Err(PaintError::InvalidData(
                "color must hex format RGBA".to_owned(),
            )),
        }
    }
}

pub const BLOCK_BITS: usize = 4;
pub const BLOCK_SIZE: usize = 1 << BLOCK_BITS;

pub struct RGBBlock {
    pixels: Box<[u8; 3 * BLOCK_SIZE * BLOCK_SIZE]>,
}

fn mix(c1: u8, c2: u8, a: u8) -> u8 {
    if a == 255 {
        c2
    } else {
        let x = (c2 as u32) * (a as u32) + (c1 as u32) * (255 - a as u32);
        let mut y = x / 255;
        if x % 255 > 255 / 2 {
            y += 1;
        }
        y as u8
    }
}

fn pos(x: u8, y: u8) -> usize {
    (BLOCK_SIZE - 1 - (y as usize)) * BLOCK_SIZE + (x as usize)
}

impl RGBBlock {
    pub fn draw_pixels<I>(&mut self, rgba: RGBA, offsets: I)
    where
        I: IntoIterator<Item = Offset>,
    {
        for (x, y) in offsets {
            let idx = 3 * pos(x, y);
            for k in 0..3 {
                self.pixels[idx + k] = mix(self.pixels[idx + k], rgba.0[k], rgba.0[3]);
            }
        }
    }

    pub fn draw_block(&mut self, blk: &RGBABlock) {
        for i in 0..BLOCK_SIZE {
            for j in 0..BLOCK_SIZE {
                let idx = pos(i as u8, j as u8);
                let idx1 = 3 * idx;
                let idx2 = 4 * idx;
                let a = blk.pixels[idx2 + 3];
                for k in 0..3 {
                    self.pixels[idx1 + k] = mix(self.pixels[idx1 + k], blk.pixels[idx2 + k], a);
                }
            }
        }
    }

    pub fn store_png<W: Write>(&self, w: W) -> Result<(), InternalError> {
        use png::{BitDepth, ColorType};
        let mut encoder = png::Encoder::new(w, BLOCK_SIZE as u32, BLOCK_SIZE as u32);
        encoder.set_color(ColorType::RGB);
        encoder.set_depth(BitDepth::Eight);
        let mut write = encoder.write_header()?;
        write.write_image_data(self.pixels.as_ref())?;
        Ok(())
    }
}

impl Default for RGBBlock {
    fn default() -> Self {
        Self {
            pixels: Box::new([255; 3 * BLOCK_SIZE * BLOCK_SIZE]),
        }
    }
}

pub struct RGBABlock {
    pixels: Box<[u8; 4 * BLOCK_SIZE * BLOCK_SIZE]>,
}

impl RGBABlock {
    #[inline]
    pub fn new() -> Self {
        RGBABlock {
            pixels: Box::new([0u8; 4 * BLOCK_SIZE * BLOCK_SIZE]),
        }
    }
    pub fn from_png<R: Read>(r: R) -> PaintResult<Self> {
        let mut this = Self::new();
        this.load_png(r)?;
        Ok(this)
    }
    pub fn load_png<R: Read>(&mut self, r: R) -> PaintResult<()> {
        use png::{BitDepth, ColorType};
        let decoder = png::Decoder::new(r);
        let (info, mut reader) = decoder.read_info()?;

        // validate the png image
        if info.width != BLOCK_SIZE as u32 || info.height != BLOCK_SIZE as u32 {
            return Err(PaintError::InvalidPNG(format!(
                "size must be {} x {}",
                BLOCK_SIZE, BLOCK_SIZE
            )));
        }
        if info.color_type != ColorType::RGBA {
            return Err(PaintError::InvalidPNG("color type must be RGBA".to_owned()));
        }
        if info.bit_depth != BitDepth::Eight {
            return Err(PaintError::InvalidPNG("bit depth must 8".to_owned()));
        }

        reader.next_frame(self.pixels.as_mut())?;
        Ok(())
    }
}

pub struct BlockInfo {
    data: RGBBlock,
    owner: Username,
    mtime: u64,
}

impl BlockInfo {
    pub fn new() -> Self {
        Self {
            data: RGBBlock::default(),
            owner: "".to_owned(),
            mtime: 0,
        }
    }

    fn accessable(&self, user: &str) -> bool {
        self.owner == "" || self.owner == user
    }

    pub fn block_to_png<W: Write>(&self, dst: W, ts: u64) -> PaintResult<u64> {
        if self.mtime > ts {
            self.data.store_png(dst)?;
        }
        Ok(self.mtime)
    }

    pub fn draw_pixels<I>(&mut self, user: &str, rgba: RGBA, offsets: I) -> bool
    where
        I: IntoIterator<Item = Offset>,
    {
        if self.accessable(user) {
            self.data.draw_pixels(rgba, offsets);
            self.mtime = now();
            return true;
        }
        false
    }

    #[allow(dead_code)]
    pub fn draw_block(&mut self, user: &str, blk: &RGBABlock) -> bool {
        if self.accessable(user) {
            self.data.draw_block(blk);
            self.mtime = now();
            return true;
        }
        false
    }

    pub fn draw_block_from_png<R: Read>(&mut self, user: &str, src: R) -> PaintResult<bool> {
        if self.accessable(user) {
            let blk = RGBABlock::from_png(src)?;
            self.data.draw_block(&blk);
            self.mtime = now();
            return Ok(true);
        }
        Ok(false)
    }

    pub fn set_owner(&mut self, user: Username) -> bool {
        if self.accessable(&user) {
            self.owner = user;
            return true;
        }
        false
    }

    pub fn get_owner(&self) -> Username {
        self.owner.clone()
    }

    pub fn reset_owner(&mut self, user: &str) -> bool {
        if self.accessable(&user) {
            self.owner = "".to_owned();
            return true;
        }
        false
    }
}

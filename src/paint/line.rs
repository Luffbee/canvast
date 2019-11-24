use super::data::{Delta, PixelPos};

pub struct LineIter {
    end: PixelPos,
    cur: PixelPos,
    dx: i16,
    dy: i16,
    sx: i16,
    sy: i16,
    err: i16,
}

impl LineIter {
    pub fn new(start: PixelPos, delta: Delta) -> Self {
        let (dx, dy) = (delta.x.abs(), -delta.y.abs());
        let (sx, sy) = (delta.x.signum(), delta.y.signum());
        Self {
            end: start + delta,
            cur: start,
            dx,
            dy,
            sx,
            sy,
            err: dx + dy,
        }
    }
    fn next_pixel(&mut self) -> Option<PixelPos> {
        let ret = self.cur;
        if self.cur == self.end {
            return None;
        }
        let e2 = self.err * 2;
        if e2 > self.dy {
            self.err += self.dy;
            self.cur.x += self.sx as i64;
        }
        if e2 < self.dx {
            self.err += self.dx;
            self.cur.y += self.sy as i64;
        }
        Some(ret)
    }
}

impl Iterator for LineIter {
    type Item = PixelPos;
    fn next(&mut self) -> Option<PixelPos> {
        self.next_pixel()
    }
}

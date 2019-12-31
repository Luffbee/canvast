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
            self.cur.x = self.cur.x.wrapping_add(self.sx as i64);
        }
        if e2 < self.dx {
            self.err += self.dx;
            self.cur.y = self.cur.y.wrapping_add(self.sy as i64);
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

#[cfg(test)]
mod tests {
    use super::*;
    fn check_line<I: IntoIterator<Item = (i64, i64)>>(start: PixelPos, delta: Delta, ans: I) {
        let out: Vec<PixelPos> = LineIter::new(start, delta).collect();
        let ans: Vec<(i64, i64)> = ans.into_iter().collect();
        assert_eq!(out.len(), ans.len());
        for i in 0..ans.len() {
            assert_eq!(out[i].x, ans[i].0);
            assert_eq!(out[i].y, ans[i].1);
        }
    }
    #[test]
    fn test_line() {
        let path2_5 = vec![(0i64, 0i64), (0, 1), (1, 2), (1, 3), (2, 4)];
        for sgn0 in [1i64, -1i64].iter() {
            for sgn1 in [1i64, -1i64].iter() {
                let path2_5: Vec<_> = path2_5
                    .iter()
                    .map(|xy| (sgn0 * xy.0, sgn1 * xy.1))
                    .collect();
                let path5_2: Vec<_> = path2_5.iter().map(|xy| (xy.1, xy.0)).collect();
                for x in -50..=50 {
                    for y in -50..=50 {
                        check_line(
                            PixelPos { x, y },
                            Delta {
                                x: *sgn0 as i16 * 2,
                                y: *sgn1 as i16 * 5,
                            },
                            path2_5.iter().map(|xy| (xy.0 + x, xy.1 + y)),
                        );
                        check_line(
                            PixelPos { x, y },
                            Delta {
                                x: *sgn1 as i16 * 5,
                                y: *sgn0 as i16 * 2,
                            },
                            path5_2.iter().map(|xy| (xy.0 + x, xy.1 + y)),
                        );
                        check_line(
                            PixelPos { x, y },
                            Delta { x: 0, y: 1 },
                            [(x, y)].iter().copied(),
                        )
                    }
                }
            }
        }
    }

    #[test]
    fn test_overflow() {
        use std::i64;
        check_line(
            PixelPos {
                x: i64::MAX,
                y: i64::MAX,
            },
            Delta { x: 2, y: 2 },
            [(i64::MAX, i64::MAX), (i64::MIN, i64::MIN)].iter().copied(),
        );
        check_line(
            PixelPos {
                x: i64::MIN,
                y: i64::MIN,
            },
            Delta { x: -2, y: -2 },
            [(i64::MIN, i64::MIN), (i64::MAX, i64::MAX)].iter().copied(),
        );
    }
}

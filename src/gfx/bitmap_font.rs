use super::draw_target::DrawTarget;



pub struct Glyph {
    pub width: usize,
    pub height: usize,
    pub bitmap: &'static [u8],
    pub baseline: isize,
    pub advance: usize,
}

impl Glyph {
    pub fn draw<Target: DrawTarget>(&self, target: &mut Target, position: (isize, isize), color: bool) {
        for gy in 0..self.height {
            for gx in 0..self.width {
                let bit = gx + gy * self.width;
                let byte = bit / 8;
                let bit = bit % 8;
                if (self.bitmap[byte] & (1 << bit)) != 0 {
                    let position = (
                        position.0 + gx as isize,
                        position.1 + gy as isize - self.baseline
                    );
                    target.set_pixel(position, color);
                }
            }
        }
    }

    pub fn advance(&self) -> usize {
        self.advance
    }
}

pub struct GlyphRange {
    pub start_char: char,
    pub glyphs: &'static [&'static Glyph]
}

pub struct BitmapFont {
    pub line_height: usize,
    pub ranges: &'static [&'static GlyphRange],
}

impl BitmapFont {
    pub fn find_glyph(&self, c: char) -> Option<&'static Glyph> {
        for range in self.ranges {
            if (range.start_char as usize) <= (c as usize) && (c as usize) < (range.start_char as usize + range.glyphs.len()) {
                let index = c as usize - range.start_char as usize;
                return Some(range.glyphs[index]);
            }
        }
        None
    }

    pub fn draw_text_line<Target: DrawTarget>(&self, target: &mut Target, baseline_position: (isize, isize), line: &str, color: bool) {
        let mut position = baseline_position;
        for c in line.chars() {
            if let Some(glyph) = self.find_glyph(c) {
                glyph.draw(target, position, color);
                position.0 += glyph.advance() as isize;
            }
        }
    }

    pub fn get_text_width(&self, line: &str) -> isize {
        let mut width = 0;
        for c in line.chars() {
            if let Some(glyph) = self.find_glyph(c) {
                width += glyph.advance() as isize;
            }
        }
        width.saturating_sub(1)
    }
}




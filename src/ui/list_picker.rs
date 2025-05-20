use alloc::vec::Vec;

use crate::{application::EncoderState, gfx::{draw_target::{DrawTarget, MaskedDrawTarget, RectMask, TranslatedDrawTarget, _DTRef, _Maskable, _Translatable}, fonts::BASIC_5PX, primitives::*}, mn12864k::Framebuffer};

pub struct ListPicker<T: Clone, const N: usize> {
    items: [(T, &'static str); N],
    index: usize,
    position: (isize, isize),
    selected: bool,
    width: usize,
    height: usize,
    scroll_offset: isize,
}

impl<T: Clone, const N: usize> ListPicker<T, N> {
    pub fn new(items: [(T, &'static str); N], position: (isize, isize), width: usize, height: usize) -> Self {
        Self {
            items,
            index: 0,
            position,
            width,
            height,
            selected: false,
            scroll_offset: 0,
        }
    }

    pub fn reset(&mut self) {
        self.index = 0;
        self.selected = false;
    }

    pub fn update(&mut self, encoder: &EncoderState) -> Option<T> {
        let mut selected = None;

        match self.selected {
            false => {
                self.index = (self.index as i32 + encoder.delta).clamp(0, self.items.len() as i32 - 1) as usize;
                if encoder.button.pressed {
                    self.selected = true;
                }
            },
            true => {
                if encoder.button.released {
                    selected = Some(self.items[self.index].0.clone());
                    self.selected = false;
                } else {
                    if encoder.delta != 0 {
                        self.selected = false;
                        self.index = (self.index as i32 + encoder.delta).clamp(0, self.items.len() as i32 - 1) as usize;
                    }
                }
            }
        }
        selected
    }

    pub fn render(&mut self, framebuffer: &mut Framebuffer) {
        let total_height = N * 11;

        let min_selected_y = self.scroll_offset + self.index as isize * 10;
        let max_selected_y = self.scroll_offset + self.index as isize * 10 + 10;

        if min_selected_y < 0 {
            self.scroll_offset += -min_selected_y;
        } else if max_selected_y > self.height as isize {
            self.scroll_offset -= max_selected_y - self.height as isize;
        }

        let mut mask = RectMask {
            upper_left: (0, 0),
            lower_right: (self.width as isize, self.height as isize)
        };
        let mut translated_framebuffer = framebuffer.dt_ref()
            .translate((self.position.0, self.position.1));

        draw_hline(&mut translated_framebuffer, 0, (self.width - 1) as isize, 0, true);
        draw_hline(&mut translated_framebuffer, 0, (self.width - 1) as isize, self.height as isize, true);
        draw_vline(&mut translated_framebuffer, 0, 0, self.height as isize, true);
        draw_vline(&mut translated_framebuffer, self.width as isize, 0, self.height as isize, true);

        let mut target = translated_framebuffer
            .mask(mask)
            .translate((0, self.scroll_offset));

        for i in 1..self.items.len() {
            draw_hline(&mut target, 0, (self.width - 1) as isize, i as isize * 10, true);
        }
        for i in 0..self.items.len() {
            if self.index == i {
                match self.selected {
                    true => {
                        draw_filled_rect(&mut target, (1, 1 + (i as isize * 10)), (self.width as isize - 1, 9 + (i as isize * 10)), true);
                        BASIC_5PX.draw_text_line(&mut target, (3, 7 + (i as isize * 10)), self.items[i].1, false);
                    }
                    false => {
                        draw_rect(&mut target, (1, 1 + (i as isize * 10)), (self.width as isize - 1, 9 + (i as isize * 10)), true);
                        BASIC_5PX.draw_text_line(&mut target, (3, 7 + (i as isize * 10)), self.items[i].1, true);
                    }
                }
            } else {
                BASIC_5PX.draw_text_line(&mut target, (3, 7 + (i as isize * 10)), self.items[i].1, true);
            }
        }
    }

    pub fn selected(&self) -> &T {
        &self.items[self.index].0
    }
}

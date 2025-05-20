use crate::application::{AppSharedState, ComState, InputState};
use crate::gfx::fonts::BASIC_5PX;
use crate::gfx::primitives::*;
use crate::mn12864k::Framebuffer;

mod phase_tuning;
mod view_picker;
mod ping_test;
mod debug_led;
mod stat_monitor;
mod open_loop_test;

pub use phase_tuning::PhaseTuningView;
pub use view_picker::ViewPickerView;
pub use ping_test::PingTestView;
pub use debug_led::DebugLedView;
pub use stat_monitor::StatMonitorView;
pub use open_loop_test::OpenLoopTestView;

#[derive(Copy, Clone, Debug)]
pub enum View {
    ViewPicker,
    PingTest,
    PhaseTuning,
    DebugLed,
    StatMonitor,
    OpenLoopTest,
}

pub trait AppView {
    fn start(&mut self);
    fn update(&mut self, dt_micros: u64, input_state: InputState, com: &mut ComState<'_>, shared_state: &mut AppSharedState) -> Option<View>;
    fn render(&mut self, framebuffer: &mut Framebuffer, shared_state: &mut AppSharedState);
}

pub struct UiFrameButton {
    pub text: &'static str,
    pub press: bool,
    pub down: bool,
}

impl UiFrameButton {
    pub fn new(text: &'static str) -> Self {
        Self {
            text,
            press: false,
            down: false,
        }
    }

    pub fn reset(&mut self) {
        self.press = false;
        self.down = false;
    }
}

pub fn update_app_frame(input_state: &InputState, buttons: &mut [UiFrameButton]) {
    if buttons.len() > 0 {
        buttons[0].down = input_state.buttons[0].down;
        buttons[0].press = input_state.buttons[0].released;
    }
    if buttons.len() > 1 {
        buttons[1].down = input_state.buttons[1].down;
        buttons[1].press = input_state.buttons[1].released;
    }
    if buttons.len() > 2 {
        buttons[2].down = input_state.buttons[2].down;
        buttons[2].press = input_state.buttons[2].released;
    }
}

pub fn render_app_frame(framebuffer: &mut Framebuffer, title: &'static str, buttons: &mut [UiFrameButton]) {
    draw_rect(framebuffer, (0, 0), (127, 63), true);
    draw_hline(framebuffer, 1, 126, 10, true);
    if buttons.len() != 0 {
        draw_hline(framebuffer, 1, 126, 53, true);
        draw_vline(framebuffer, 32, 54, 62, true);
        if buttons[0].down {
            draw_filled_rect(framebuffer, (1, 54), (31, 62), true);
            BASIC_5PX.draw_text_line(framebuffer, (3,60), buttons[0].text, false);
        } else {
            BASIC_5PX.draw_text_line(framebuffer, (3,60), buttons[0].text, true);
        }
        if buttons.len() > 1 {
            draw_vline(framebuffer, 64, 54, 62, true);
            if buttons[1].down {
                draw_filled_rect(framebuffer, (33, 54), (63, 62), true);
                BASIC_5PX.draw_text_line(framebuffer, (35,60), buttons[1].text, false);
            } else {
                BASIC_5PX.draw_text_line(framebuffer, (35,60), buttons[1].text, true);
            }
        }
        if buttons.len() > 2 {
            draw_vline(framebuffer, 96, 54, 62, true);
            if buttons[2].down {
                draw_filled_rect(framebuffer, (65, 54), (95, 62), true);
                BASIC_5PX.draw_text_line(framebuffer, (67,60), buttons[2].text, false);
            } else {
                BASIC_5PX.draw_text_line(framebuffer, (67,60), buttons[2].text, true);
            }
        }
    }
    BASIC_5PX.draw_text_line(framebuffer, (3, 7), title, true);
}

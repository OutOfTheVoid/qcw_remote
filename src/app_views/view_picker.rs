use alloc::format;

use crate::{application::{AppSharedState, ComState, InputState}, gfx::fonts::BASIC_5PX, ui::ListPicker, mn12864k::Framebuffer};

use super::{render_app_frame, AppView, View};

pub struct ViewPickerView {
    picker: ListPicker<View, 5>
}

impl ViewPickerView {
    pub fn new() -> Self {
        Self {
            picker: ListPicker::new([
                (View::DebugLed, "Debug LED Control"),
                (View::PingTest, "Ping Test"),
                (View::PhaseTuning, "Feedback Phase Tuning"),
                (View::StatMonitor, "Stat Monitor"),
                (View::OpenLoopTest, "Open Loop Test"),
            ], (2, 20), 122, 40),
        }
    }
}

impl AppView for ViewPickerView {
    fn start(&mut self) {
        self.picker.reset();
    }

    fn update(&mut self, dt_micros: u64, input_state: InputState, com: &mut ComState<'_>, shared_state: &mut AppSharedState) -> Option<super::View> {
        com.inbox.clear();
        self.picker.update(&input_state.encoder)
    }

    fn render(&mut self, framebuffer: &mut Framebuffer, shared_state: &mut AppSharedState) {
        render_app_frame(framebuffer, "Home", &mut []);
        BASIC_5PX.draw_text_line(framebuffer, (5, 17), "Open Tool:", true);
        self.picker.render(framebuffer);
    }
}



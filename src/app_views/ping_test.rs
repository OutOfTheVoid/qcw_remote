use alloc::format;
use alloc::vec::Vec;
use crate::app_views::AppView;
use crate::application::AppSharedState;
use crate::application::ComState;
use crate::application::InputState;
use crate::gfx::fonts::BASIC_5PX;
use crate::mn12864k::Framebuffer;
use qcw_com::{ControllerMessage, RemoteMessage};

use super::render_app_frame;
use super::update_app_frame;
use super::UiFrameButton;
use super::View;

pub struct PingTestView {
    t: u64,
    seq: u32,
    time_last_sent: u64,
    sent_seq: u32,
    received_seq: Option<u32>,
    buttons: [UiFrameButton; 1],
}

impl PingTestView {
    pub fn new() -> Self {
        Self {
            t: 0,
            seq: 0,
            time_last_sent: 0,
            sent_seq: 0,
            received_seq: None,
            buttons: [UiFrameButton::new("Back")],
        }
    }
}

impl AppView for PingTestView {
    fn start(&mut self) {
        self.t = 0;
        self.seq = 0;
        self.time_last_sent = 0;
        self.sent_seq = 0;
        self.buttons.iter_mut().for_each(|button| button.reset());
    }

    fn update(&mut self, dt_micros: u64, input_state: InputState, com: &mut ComState<'_>, shared_state: &mut AppSharedState) -> Option<View> {
        update_app_frame(&input_state, &mut self.buttons);
        self.t += dt_micros;
        let time_since_send = self.t - self.time_last_sent;
        if time_since_send > 100000 {
            self.sent_seq = self.seq & 0x0FFFFFFF;
            self.seq = self.seq.wrapping_add(0x00010101);
            self.received_seq = None;
            com.outbox.push_back(ControllerMessage::Ping(self.sent_seq));
            self.time_last_sent = self.t;
        }
        if let Some(RemoteMessage::Ping(seq)) = com.inbox.pop_front() {
            self.received_seq = Some(seq);
        }
        if self.buttons[0].press {
            Some(View::ViewPicker)
        } else {
            None
        }
    }

    fn render(&mut self, framebuffer: &mut Framebuffer, shared_state: &mut AppSharedState) {
        render_app_frame(framebuffer, "Fiber Ping Test", &mut self.buttons);
        BASIC_5PX.draw_text_line(framebuffer, (5, 16), &format!("Tx: {:08x}", self.sent_seq), true);
        if let Some(seq) = &self.received_seq {
            BASIC_5PX.draw_text_line(framebuffer, (5, 25), &format!("Rx: {:08x}", *seq), true);
        } else {
            BASIC_5PX.draw_text_line(framebuffer, (5, 25), "Rx: --", true);
        }
    }
}

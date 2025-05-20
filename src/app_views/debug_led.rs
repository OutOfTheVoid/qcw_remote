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

pub struct DebugLedView {
    state: Option<bool>,
    buttons: [UiFrameButton; 2],
}

impl DebugLedView {
    pub fn new() -> Self {
        Self {
            state: None,
            buttons: [UiFrameButton::new("Back"), UiFrameButton::new("On")],
        }
    }
}

impl AppView for DebugLedView {
    fn start(&mut self) {
        self.state = None;
        self.buttons.iter_mut().for_each(|button| button.reset());
    }

    fn update(&mut self, dt_micros: u64, input_state: InputState, com: &mut ComState<'_>, shared_state: &mut AppSharedState) -> Option<View> {
        update_app_frame(&input_state, &mut self.buttons);
        if self.buttons[1].press {
            match self.state.clone() {
                None => {
                    self.state = Some(true);
                    com.outbox.push_back(ControllerMessage::SetDebugLed(true));
                    self.buttons[1].text = "Off";
                },
                Some(state) => {
                    self.state = Some(!state);
                    com.outbox.push_back(ControllerMessage::SetDebugLed(!state));
                    self.buttons[1].text = if !state { "LED Off" } else { "LED On" };
                }
            }
        }
        if self.buttons[0].press {
            Some(View::ViewPicker)
        } else {
            None
        }
    }

    fn render(&mut self, framebuffer: &mut Framebuffer, shared_state: &mut AppSharedState) {
        render_app_frame(framebuffer, "Debug LED Control", &mut self.buttons);
    }
}

use alloc::format;
use qcw_com::{ControllerMessage, RemoteMessage, Statistic, StatisticValue};

use crate::{application::{AppSharedState, ComState, InputState}, gfx::fonts::BASIC_5PX, mn12864k::Framebuffer};

use super::{render_app_frame, update_app_frame, AppView, UiFrameButton, View};

pub struct StatMonitorView {
    buttons: [UiFrameButton; 2],
    t_elapsed: u64,
    t_last_request: u64,
    max_current_value: f32,
    feedback_frequency_value: f32,
}

impl StatMonitorView {
    pub fn new() -> Self {
        Self {
            buttons: [UiFrameButton::new("Back"), UiFrameButton::new("Reset")],
            t_elapsed: 0,
            t_last_request: 0,
            max_current_value: 0.0,
            feedback_frequency_value: 0.0,
        }
    }
}

impl AppView for StatMonitorView {
    fn start(&mut self) {
        self.t_elapsed = 0;
        self.t_last_request = 0;
        self.max_current_value = 0.0;
        self.feedback_frequency_value = 0.0;
    }

    fn update(&mut self, dt_micros: u64, input_state: InputState, com: &mut ComState<'_>, shared_state: &mut AppSharedState) -> Option<super::View> {
        self.t_elapsed += dt_micros;
        // every 30 ms, request the value of the max primary current
        if self.t_elapsed - self.t_last_request >= 30000 {
            com.outbox.push_back(ControllerMessage::GetStat(Statistic::MaxPrimaryCurrent));
            com.outbox.push_back(ControllerMessage::GetStat(Statistic::FeedbackFrequency));
            self.t_last_request = self.t_elapsed;
        }
        while let Some(message) = com.inbox.pop_front() {
            match message {
                RemoteMessage::GetStatResult(stat) => {
                    match stat {
                        StatisticValue::MaxPrimaryCurrentA(current) => self.max_current_value = current,
                        StatisticValue::FeedbackFrequencykHz(frequency) => self.feedback_frequency_value = frequency,
                    }
                },
                _ => {}
            }
        }
        update_app_frame(&input_state, &mut self.buttons);
        if self.buttons[1].press {
            com.outbox.push_back(ControllerMessage::ResetStats);
        }
        if self.buttons[0].press {
            Some(View::ViewPicker)
        } else {
            None
        }
    }

    fn render(&mut self, framebuffer: &mut Framebuffer, shared_state: &mut AppSharedState) {
        render_app_frame(framebuffer, "Home", &mut self.buttons);
        BASIC_5PX.draw_text_line(framebuffer, (4, 20), &format!("Max Current:"), true);
        BASIC_5PX.draw_text_line(framebuffer, (80, 20), &format!("{:.2} A", self.max_current_value), true);
        BASIC_5PX.draw_text_line(framebuffer, (4, 28), &format!("Feedback Frequency:"), true);
        BASIC_5PX.draw_text_line(framebuffer, (80, 28), &format!("{:.2} kHz", self.feedback_frequency_value), true);
    }
}



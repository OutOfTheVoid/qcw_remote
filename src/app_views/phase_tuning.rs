use alloc::format;
use crate::application::{AppSharedState, ComState, InputState};
use crate::gfx::primitives::*;
use crate::gfx::fonts::BASIC_5PX;
use crate::mn12864k::Framebuffer;
use qcw_com::{ControllerMessage, Parameter, ParameterValue, RemoteMessage};

use super::{render_app_frame, update_app_frame, AppView, UiFrameButton, View};

enum PhaseTuningState {
    Init,
    AwaitingParams,
    RunningDisabled,
    RunningEnabled,
    Disabling(bool),
}

pub struct PhaseTuningView {
    buttons: [UiFrameButton; 3],
    state: PhaseTuningState,
    phase_delay: i16,
    delay_dirty: bool,
    t_last_keepalive: u64,
    t_elapsed: u64,
}

const TUNING_RANGE: i16 = 400;
const KEEPALIVE_INTERVAL_US: u64 = 10_000;

impl PhaseTuningView {
    pub fn new() -> Self {
        Self {
            buttons: [UiFrameButton::new("Back"), UiFrameButton::new("Run"), UiFrameButton::new("Reset")],
            state: PhaseTuningState::RunningDisabled,
            phase_delay: 0,
            delay_dirty: false,
            t_elapsed: 0,
            t_last_keepalive: 0,
        }
    }
}

impl AppView for PhaseTuningView {
    fn start(&mut self) {
        self.buttons.iter_mut().for_each(|button| button.reset());
        self.state = PhaseTuningState::Init;
        self.buttons[1].text = "---";
        self.phase_delay = 0;
        self.delay_dirty = false;
    }

    fn update(&mut self, dt_micros: u64, input_state: InputState, com: &mut ComState<'_>, shared_state: &mut AppSharedState) -> Option<View> {
        self.t_elapsed += dt_micros;
        update_app_frame(&input_state, &mut self.buttons);
        let control_enabled = match self.state {
            PhaseTuningState::Init => {
                self.state = PhaseTuningState::AwaitingParams;
                self.buttons[1].text = "--";
                com.outbox.push_back(ControllerMessage::GetParam(Parameter::DelayCompensation));
                false
            },
            PhaseTuningState::AwaitingParams => {
                while let Some(message) = com.inbox.pop_front() {
                    match message {
                        RemoteMessage::GetParamResult(ParameterValue::DelayCompensationNS(value)) => {
                            self.phase_delay = value;
                            self.state = PhaseTuningState::RunningDisabled;
                            self.buttons[1].text = "Run";
                            self.buttons[2].text = "Reset";
                        },
                        _ => {}
                    }
                }
                false
            },
            PhaseTuningState::RunningDisabled => {
                if self.buttons[2].press {
                    self.buttons[1].text = "---";
                    self.state = PhaseTuningState::Init;
                }
                if self.buttons[1].press {
                    com.outbox.push_back(ControllerMessage::SetParam(ParameterValue::FlatPower(1.0)));
                    com.outbox.push_back(ControllerMessage::SetParam(ParameterValue::OnTimeUs(600)));
                    com.outbox.push_back(ControllerMessage::SetParam(ParameterValue::OffTimeMs(300)));
                    com.outbox.push_back(ControllerMessage::SetParam(ParameterValue::RunMode(qcw_com::RunMode::TestClosedLoop)));
                    com.outbox.push_back(ControllerMessage::Run);
                    self.buttons[1].text = "Stop";
                    self.state = PhaseTuningState::RunningEnabled;
                }
                true
            },
            PhaseTuningState::RunningEnabled => {
                if self.buttons[2].press {
                    self.buttons[1].text = "---";
                    self.state = PhaseTuningState::Disabling(true);
                    false
                } else if self.buttons[1].press {
                    self.buttons[1].text = "---";
                    self.state = PhaseTuningState::Disabling(false);
                    false
                } else {
                    if self.t_elapsed - self.t_last_keepalive >= KEEPALIVE_INTERVAL_US {
                        com.outbox.push_back(ControllerMessage::KeepAlive);
                        self.t_last_keepalive = self.t_elapsed;
                    }
                    true
                }
            },
            PhaseTuningState::Disabling(reinit) => {
                com.outbox.push_back(ControllerMessage::Stop);
                if reinit {
                    self.buttons[1].text = "---";
                    self.state = PhaseTuningState::Init;
                } else {
                    self.buttons[1].text = "Run";
                    self.state = PhaseTuningState::RunningDisabled;
                }
                false
            }
        };
        if control_enabled {
            let old_phase_delay = self.phase_delay;
            self.phase_delay = self.phase_delay.saturating_add(input_state.encoder.delta as i16);
            self.phase_delay = self.phase_delay.clamp(-TUNING_RANGE, TUNING_RANGE);
            if input_state.encoder.button.pressed {
                self.phase_delay = 0;
            }
            if self.phase_delay != old_phase_delay {
                self.delay_dirty = true;
                com.outbox.push_back(ControllerMessage::SetParam(ParameterValue::DelayCompensationNS(self.phase_delay)));
                com.outbox.push_back(ControllerMessage::GetParam(Parameter::DelayCompensation));
            }
            while let Some(message) = com.inbox.pop_front() {
                match message {
                    RemoteMessage::GetParamResult(ParameterValue::DelayCompensationNS(value)) => {
                        self.phase_delay = value as i16;
                        self.delay_dirty = false;
                    },
                    _ => {}
                }
            }
        }
        if self.buttons[0].press {
            com.outbox.push_back(qcw_com::ControllerMessage::Stop);
            Some(View::ViewPicker)
        } else {
            None
        }
    }

    fn render(&mut self, framebuffer: &mut Framebuffer, shared_state: &mut AppSharedState) {
        render_app_frame(framebuffer, "Feedback Phase Tuning", &mut self.buttons);

        let state_string = match self.state {
            PhaseTuningState::Init => "Initializing...",
            PhaseTuningState::AwaitingParams => "Waiting for controller...",
            PhaseTuningState::RunningDisabled => "Disabled",
            PhaseTuningState::RunningEnabled => "Enabled",
            PhaseTuningState::Disabling(_) => "Disabling...",
        };
        BASIC_5PX.draw_text_line(framebuffer, (4, 20), &format!("        State: {}", state_string), true);

        let phase_delay_string = format!("Phase Delay: {}ns", self.phase_delay);
        BASIC_5PX.draw_text_line(framebuffer, (4, 28), &phase_delay_string, true);
        draw_hline(framebuffer, BASIC_5PX.get_text_width("Phase Delay: ") + 4, BASIC_5PX.get_text_width(&phase_delay_string) + 4, 30, true);

        draw_hline(framebuffer, 63-40, 63, 40, true);
        draw_hline(framebuffer, 64, 63+40, 32, true);
        draw_line(framebuffer, (63, 40), (63, 32), true);

        let cursor_x = 63 + (self.phase_delay as isize * 40) / TUNING_RANGE as isize;
        draw_line(framebuffer, (cursor_x - 2, 45), (cursor_x, 43), true);
        draw_line(framebuffer, (cursor_x + 2, 45), (cursor_x, 43), true);
        draw_line(framebuffer, (cursor_x, 43), (cursor_x, 49), true);
        
        draw_hline(framebuffer, 63 - 40, 63 + 40, 49, true);
        draw_vline(framebuffer, 63-40, 48, 50, true);
        draw_vline(framebuffer, 63+40, 48, 50, true);
        draw_vline(framebuffer, 63, 48, 50, true);
    }
}

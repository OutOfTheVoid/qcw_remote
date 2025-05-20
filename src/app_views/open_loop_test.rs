use alloc::{format, string::String};
use libm::roundf;
use qcw_com::{Parameter, ParameterValue, RemoteMessage};

use crate::{gfx::{fonts::BASIC_5PX, primitives::draw_hline}, ui::ListPicker};

use super::{render_app_frame, update_app_frame, AppView, UiFrameButton};

const PARAMETER_COUNT: usize = 4;

const PARAMETER_REQUEST_INTERVAL_US: u64 = 100_000;
const KEEPALIVE_INTERVAL_US: u64 = 10_000;

pub struct OpenLoopTestView {
    frame_buttons: [UiFrameButton; 2],
    editing: bool,
    running: bool,
    t_last_keepalive: u64,
    t_last_getparams: u64,
    t_elapsed: u64,
    on_time: u32,
    off_time: u32,
    frequency: u32,
    power: u32,
    parameter_list: ListPicker<Parameter, PARAMETER_COUNT>,
}

impl OpenLoopTestView {
    pub fn new() -> Self {
        Self {
            frame_buttons: [
                UiFrameButton::new("Back"), UiFrameButton::new("Run"), 
            ],
            editing: false,
            running: false,
            t_elapsed: 0,
            t_last_keepalive: 0,
            t_last_getparams: 0,
            on_time: 100,
            off_time: 100,
            frequency: 400,
            power: 0,
            parameter_list: ListPicker::new([
                (Parameter::OnTime, "On Time"),
                (Parameter::OffTime, "Off Time"),
                (Parameter::StartupFrequency, "Frequency"),
                (Parameter::FlatPower, "Power"),
            ], (4, 20), 45, 30),
        }
    }
}

impl AppView for OpenLoopTestView {
    fn start(&mut self) {
        self.frame_buttons.iter_mut().for_each(|button| button.reset());
        self.frame_buttons[1].text = "Run";
        self.running = false;
        self.editing = false;
        self.t_elapsed = 0;
        self.t_last_keepalive = 0;
        self.t_last_getparams = 0;
    }

    fn update(&mut self, dt_micros: u64, input_state: crate::application::InputState, com: &mut crate::application::ComState<'_>, shared_state: &mut crate::application::AppSharedState) -> Option<super::View> {
        self.t_elapsed += dt_micros;
        update_app_frame(&input_state, &mut self.frame_buttons);

        if !self.editing {
            self.editing = self.parameter_list.update(&input_state.encoder).is_some();
        } else {
            if input_state.encoder.button.pressed {
                self.editing = false;
            }
        }

        while let Some(message) = com.inbox.pop_front() {
            match message {
                RemoteMessage::GetParamResult(param_value) => {
                    match param_value {
                        ParameterValue::OnTimeUs(on_time) => self.on_time = on_time as u32,
                        ParameterValue::OffTimeMs(off_time) => self.off_time = off_time as u32,
                        ParameterValue::StartupFrequencykHz(frequency) => self.frequency = frequency as u32,
                        ParameterValue::FlatPower(power) => self.power = (roundf(power * 100.0) as i32).clamp(0, 100) as u32,
                        _ => {}
                    }
                },
                _ => {}
            }
        }

        if self.editing && input_state.encoder.delta != 0 {
            match self.parameter_list.selected() {
                Parameter::OnTime => {
                    self.on_time = (self.on_time as i32 + input_state.encoder.delta * 10).clamp(0, 1000) as u32;
                    com.outbox.push_back(qcw_com::ControllerMessage::SetParam(ParameterValue::OnTimeUs(self.on_time as u16)));
                    com.outbox.push_back(qcw_com::ControllerMessage::GetParam(Parameter::OnTime));
                },
                Parameter::OffTime => {
                    self.off_time = (self.off_time as i32 + input_state.encoder.delta * 10).clamp(10, 5000) as u32;
                    com.outbox.push_back(qcw_com::ControllerMessage::SetParam(ParameterValue::OffTimeMs(self.off_time as u16)));
                    com.outbox.push_back(qcw_com::ControllerMessage::GetParam(Parameter::OffTime));
                },
                Parameter::StartupFrequency => {
                    self.frequency = (self.frequency as i32 + input_state.encoder.delta).clamp(300, 700) as u32;
                    com.outbox.push_back(qcw_com::ControllerMessage::SetParam(ParameterValue::StartupFrequencykHz(self.frequency as f32)));
                    com.outbox.push_back(qcw_com::ControllerMessage::GetParam(Parameter::StartupFrequency));
                },
                Parameter::FlatPower => {
                    self.power = (self.power as i32 + input_state.encoder.delta).clamp(0, 100) as u32;
                    com.outbox.push_back(qcw_com::ControllerMessage::SetParam(ParameterValue::FlatPower((self.power as f32) / 100.0f32)));
                    com.outbox.push_back(qcw_com::ControllerMessage::GetParam(Parameter::FlatPower));
                },
                _ => {}
            }
        }

        if (self.t_elapsed - self.t_last_getparams) >= PARAMETER_REQUEST_INTERVAL_US {
            com.outbox.push_back(qcw_com::ControllerMessage::GetParam(Parameter::OnTime));
            com.outbox.push_back(qcw_com::ControllerMessage::GetParam(Parameter::OffTime));
            com.outbox.push_back(qcw_com::ControllerMessage::GetParam(Parameter::StartupFrequency));
            com.outbox.push_back(qcw_com::ControllerMessage::GetParam(Parameter::FlatPower));
            self.t_last_getparams = self.t_elapsed;
        }

        if self.frame_buttons[1].press {
            self.running = !self.running;
            if self.running {
                com.outbox.push_back(qcw_com::ControllerMessage::SetParam(ParameterValue::RunMode(qcw_com::RunMode::OpenLoop)));
                com.outbox.push_back(qcw_com::ControllerMessage::Run);
                self.frame_buttons[1].text = "Stop";
            } else {
                com.outbox.push_back(qcw_com::ControllerMessage::Stop);
                self.frame_buttons[1].text = "Run";
            }
        }

        if self.running && ((self.t_elapsed - self.t_last_keepalive) >= KEEPALIVE_INTERVAL_US) {
            com.outbox.push_back(qcw_com::ControllerMessage::KeepAlive);
            self.t_last_keepalive = self.t_elapsed;
        }

        if self.frame_buttons[0].press {
            com.outbox.push_back(qcw_com::ControllerMessage::Stop);
            Some(super::View::ViewPicker)
        } else {
            None
        }
    }

    fn render(&mut self, framebuffer: &mut crate::mn12864k::Framebuffer, shared_state: &mut crate::application::AppSharedState) {
        render_app_frame(framebuffer, "Open Loop Test", &mut self.frame_buttons);
        BASIC_5PX.draw_text_line(framebuffer, (4, 18), "Param:", true);
        let param_string = match self.parameter_list.selected() {
            Parameter::OnTime => format!("Value: {} us", self.on_time),
            Parameter::OffTime => format!("Value: {} ms", self.off_time),
            Parameter::StartupFrequency => format!("Value: {} kHz", self.frequency),
            Parameter::FlatPower => format!("Value: {} %", self.power),
            _ => String::new(),
        };
        BASIC_5PX.draw_text_line(framebuffer, (55, 18), &param_string, true);
        if self.editing {
            let text_width = BASIC_5PX.get_text_width(&param_string);
            draw_hline(framebuffer, 55, 55 + text_width, 20, true);
        }
        BASIC_5PX.draw_text_line(framebuffer, (55, 26), if self.running { "Running" } else { "Stopped" }, true);
        self.parameter_list.render(framebuffer);
    }
}


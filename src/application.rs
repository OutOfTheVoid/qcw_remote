use alloc::collections::vec_deque::VecDeque;
use alloc::vec::Vec;

use crate::app_views::*;
use crate::mn12864k::Framebuffer;
use crate::gfx;
use qcw_com::{ControllerMessage, RemoteMessage};

pub struct ButtonState {
    pub down: bool,
    pub pressed: bool,
    pub released: bool,
}

pub struct EncoderState {
    pub count: i32,
    pub delta: i32,
    pub button: ButtonState,
}

pub struct InputState {
    pub encoder: EncoderState,
    pub buttons: [ButtonState; 3],
}

pub struct Application {
    view_picker_view: ViewPickerView,
    phase_tuning_view: PhaseTuningView,
    ping_test_view: PingTestView,
    debug_led_view: DebugLedView,
    stat_monitor_view: StatMonitorView,
    open_loop_test_view: OpenLoopTestView,
    incoming_view: Option<View>,
    current_view: Option<View>,
    shared_state: AppSharedState,
}

pub struct ComState<'a> {
    pub inbox: &'a mut VecDeque<RemoteMessage>,
    pub outbox: &'a mut VecDeque<ControllerMessage>,
}

pub struct AppSharedState {
}

impl Application {
    pub fn new(shared_state: AppSharedState) -> Self {
        Self {
            shared_state,
            view_picker_view: ViewPickerView::new(),
            phase_tuning_view: PhaseTuningView::new(),
            ping_test_view: PingTestView::new(),
            debug_led_view: DebugLedView::new(),
            stat_monitor_view: StatMonitorView::new(),
            open_loop_test_view: OpenLoopTestView::new(),
            incoming_view: Some(View::ViewPicker),
            current_view: None,
        }
    }

    pub fn update(&mut self, dt_micros: u64, input_state: InputState, mut com: ComState<'_>) {
        if let Some(incoming_view) = self.incoming_view.take() {
            let view: &mut dyn AppView = match incoming_view {
                View::ViewPicker => &mut self.view_picker_view,
                View::PingTest => &mut self.ping_test_view,
                View::PhaseTuning => &mut self.phase_tuning_view,
                View::DebugLed => &mut self.debug_led_view,
                View::StatMonitor => &mut self.stat_monitor_view,
                View::OpenLoopTest => &mut self.open_loop_test_view,
            };
            self.current_view = Some(incoming_view);
        }
        if let Some(current_view) = &self.current_view {
            let view: &mut dyn AppView = match current_view {
                View::ViewPicker => &mut self.view_picker_view,
                View::PingTest => &mut self.ping_test_view,
                View::PhaseTuning => &mut self.phase_tuning_view,
                View::DebugLed => &mut self.debug_led_view,
                View::StatMonitor => &mut self.stat_monitor_view,
                View::OpenLoopTest => &mut self.open_loop_test_view,
            };
            self.incoming_view = view.update(dt_micros, input_state, &mut com, &mut self.shared_state);
        }
    }

    pub fn render(&mut self, framebuffer: &mut Framebuffer) {
        if let Some(current_view) = &self.current_view {
            let view: &mut dyn AppView = match current_view {
                View::ViewPicker => &mut self.view_picker_view,
                View::PingTest => &mut self.ping_test_view,
                View::PhaseTuning => &mut self.phase_tuning_view,
                View::DebugLed => &mut self.debug_led_view,
                View::StatMonitor => &mut self.stat_monitor_view,
                View::OpenLoopTest => &mut self.open_loop_test_view,
            };
            view.render(framebuffer, &mut self.shared_state);
        }
    }
}

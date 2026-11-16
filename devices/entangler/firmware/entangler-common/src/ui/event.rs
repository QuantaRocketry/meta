use slint::platform::WindowEvent;

pub enum Event {
    SlintTouchEvent(WindowEvent),
    StateEvent,
}

impl From<WindowEvent> for Event {
    fn from(value: WindowEvent) -> Self {
        Self::SlintTouchEvent(value)
    }
}

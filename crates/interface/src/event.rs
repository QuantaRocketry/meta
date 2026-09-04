#[derive(Debug, Copy, Clone)]
pub enum NavigateEvent {
    Up,
    Down,
    Left,
    Right,
    Select,
}

#[derive(Debug, Copy, Clone)]
pub enum InterfaceEvent {
    Navigate(NavigateEvent),
}

impl From<NavigateEvent> for InterfaceEvent {
    fn from(value: NavigateEvent) -> Self {
        Self::Navigate(value)
    }
}

pub mod location;
pub mod settings;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum Page {
    #[default]
    Location,
    Settings,
}

impl Page {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Page::Location => "Location",
            Page::Settings => "Settings",
        }
    }
}

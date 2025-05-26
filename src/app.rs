use crate::event::EventHandler;

#[derive(Debug)]
pub struct App {
    pub wifi_enabled: bool,
    pub airplane_mode: bool,
    pub events: EventHandler,
}

impl Default for App {
    fn default() -> Self {
        Self {
            wifi_enabled: false,
            airplane_mode: false,
            events: EventHandler::new(),
        }
    }
}

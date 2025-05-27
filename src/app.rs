#[derive(Debug)]
pub struct App {
    pub wifi_enabled: bool,
    pub airplane_mode: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            wifi_enabled: false,
            airplane_mode: false,
        }
    }
}

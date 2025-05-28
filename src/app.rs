use anyhow::Result;

use crate::nm::NetworkManager;

#[derive(Debug, Clone)]
pub struct State {
    wifi_enabled: bool,
    airplane_mode: bool,
}

impl State {
    pub async fn new(nm: &NetworkManager) -> Result<Self> {
        let wifi_enabled = nm.wifi_enabled().await?;
        let airplane_mode = nm.airplane_mode_enabled().await?;

        Ok(Self {
            wifi_enabled,
            airplane_mode,
        })
    }
}

#[derive(Debug)]
pub struct App {
    state: State,
}

impl App {
    pub async fn new(nm: &NetworkManager) -> Self {
        let state = State::new(nm).await.unwrap();
        Self { state }
    }
}

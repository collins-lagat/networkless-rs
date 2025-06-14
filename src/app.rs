use std::time::Duration;

use anyhow::Result;
use ksni::Handle;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::tray::Tray;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Tick,
    Shutdown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    ChangeAccessPoint(String),
    ToggleWifi,
    ToggleWired,
    ToggleAirplaneMode,
    ToggleVPN,
}

#[derive(Debug, Clone)]
pub struct App {
    event_tx: Sender<Event>,
    action_tx: Sender<Action>,
}

impl App {
    pub fn new(event_tx: Sender<Event>, action_tx: Sender<Action>) -> Self {
        let actor = AppTicker::new(event_tx.clone());
        tokio::spawn(async move { actor.run().await });

        Self {
            event_tx,
            action_tx,
        }
    }

    pub async fn send_event(&self, event: Event) {
        self.event_tx.send(event).await.unwrap();
    }

    pub async fn send_action(&self, action: Action) {
        self.action_tx.send(action).await.unwrap();
    }

    pub fn send_action_blocking(&self, action: Action) {
        self.action_tx.blocking_send(action).unwrap();
    }

    pub async fn run(
        &self,
        mut event_rx: Receiver<Event>,
        mut action_rx: Receiver<Action>,
        tray_handle: Handle<Tray>,
    ) {
        let action_tray_handle = tray_handle.clone();
        tokio::spawn(async move {
            while let Some(event) = action_rx.recv().await {
                println!("Action: {:?}", event);
            }
        });

        while let Some(event) = event_rx.recv().await {
            println!("Event: {:?}", event);

            match event {
                Event::Tick => {
                    println!("Tick");
                }
                Event::Shutdown => break,
            }
        }
    }
}

struct AppTicker {
    tx: Sender<Event>,
}

impl AppTicker {
    fn new(tx: Sender<Event>) -> Self {
        Self { tx }
    }

    async fn run(&self) -> Result<()> {
        let tick_rate = Duration::from_secs(1);
        let mut interval = tokio::time::interval(tick_rate);
        loop {
            interval.tick().await;
            self.tx.send(Event::Tick).await?;
        }
    }
}

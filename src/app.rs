use ksni::Handle;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::tray::Tray;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Shutdown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    ToggleWifi,
    ToggleWired,
    ToggleBluetooth,
    ToggleAirplaneMode,
}

#[derive(Debug, Clone)]
pub struct App {
    tx: Sender<Event>,
}

impl App {
    pub fn new(tx: Sender<Event>) -> Self {
        Self { tx }
    }

    pub async fn run(&self, rx: &mut Receiver<Event>, tray_handle: Handle<Tray>) {
        while let Some(event) = rx.recv().await {
            println!("Event: {:?}", event);

            match event {
                Event::Shutdown => break,
                _ => {}
            }
        }
    }
}

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
    ToggleBluetooth,
    ToggleAirplaneMode,
}

pub struct App {
    tx: Sender<Event>,
    rx: Receiver<Event>,
    tray_handle: Option<Handle<Tray>>,
}

impl App {
    pub fn new(tx: Sender<Event>, rx: Receiver<Event>) -> Self {
        Self {
            tx,
            rx,
            tray_handle: None,
        }
    }

    pub fn set_tray_handle(&mut self, tray_handle: Handle<Tray>) {
        self.tray_handle = Some(tray_handle);
    }

    pub async fn run(&mut self) {
        while let Some(event) = self.rx.recv().await {
            println!("Event: {:?}", event);

            match event {
                Event::Shutdown => break,
                _ => {}
            }
        }
    }
}

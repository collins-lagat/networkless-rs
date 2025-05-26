use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

#[derive(Debug, Clone)]
pub enum Event {
    Init,
    WifiEnabled(bool),
    AirplaneMode(bool),
    Shutdown,
}

#[derive(Debug)]
pub struct EventHandler {
    tx: UnboundedSender<Event>,
    rx: UnboundedReceiver<Event>,
}

impl EventHandler {
    pub fn new() -> Self {
        let (tx, rx) = unbounded_channel();
        Self { tx, rx }
    }

    pub fn sender(&self) -> UnboundedSender<Event> {
        self.tx.clone()
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }

    pub async fn send(&mut self, event: Event) {
        self.tx.send(event);
    }
}

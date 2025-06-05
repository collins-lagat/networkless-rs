use tokio::sync::mpsc::{Receiver, Sender};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Shutdown,
}

pub struct App {
    tx: Sender<Event>,
    rx: Receiver<Event>,
}

impl App {
    pub fn new(tx: Sender<Event>, rx: Receiver<Event>) -> Self {
        Self { tx, rx }
    }

    pub async fn run(&mut self) {
        while let Some(event) = self.rx.recv().await {
            println!("{:?}", event);
        }
    }
}

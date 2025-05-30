// use futures::StreamExt;
// use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};
// use log::info;
// use signal_hook::{
//     consts::{SIGINT, SIGTERM},
//     iterator::Signals,
// };

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Unknown,
    Off,
    Busy,
    Disconnected,
    AirplaneMode,
    Limited,
    Vpn,
    Ethernet,
    Wifi(u8),

    Shutdown,
}

// #[derive(Debug)]
// pub struct EventHandler {
//     tx: UnboundedSender<Event>,
//     rx: UnboundedReceiver<Event>,
// }
//
// impl EventHandler {
//     pub fn new() -> Self {
//         let (tx, rx) = unbounded();
//
//         let mut signals = Signals::new([SIGINT, SIGTERM]).unwrap();
//
//         let mut signal_tx = tx.clone();
//         tokio::spawn(async move {
//             for signal in signals.forever() {
//                 info!("Received signal {:?}", signal);
//                 signal_tx.unbounded_send(Event::Shutdown);
//             }
//         });
//
//         Self { tx, rx }
//     }
//
//     pub async fn next(&mut self) -> Option<Event> {
//         self.rx.next().await
//     }
//
//     pub async fn send(&mut self, event: Event) {
//         let _ = self.tx.unbounded_send(event);
//     }
// }

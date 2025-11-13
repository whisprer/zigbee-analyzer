use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent};
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Clone, Copy, Debug)]
pub enum Event {
    Tick,
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
}

pub struct EventHandler {
    rx: mpsc::UnboundedReceiver<Event>,
    #[allow(dead_code)]
    tx: mpsc::UnboundedSender<Event>,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let event_tx = tx.clone();

        tokio::spawn(async move {
            let mut last_tick = tokio::time::Instant::now();
            loop {
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or(Duration::from_secs(0));

                if event::poll(timeout).unwrap() {
                    match event::read().unwrap() {
                        CrosstermEvent::Key(e) => {
                            event_tx.send(Event::Key(e)).unwrap();
                        }
                        CrosstermEvent::Mouse(e) => {
                            event_tx.send(Event::Mouse(e)).unwrap();
                        }
                        CrosstermEvent::Resize(w, h) => {
                            event_tx.send(Event::Resize(w, h)).unwrap();
                        }
                        _ => {}
                    }
                }

                if last_tick.elapsed() >= tick_rate {
                    event_tx.send(Event::Tick).unwrap();
                    last_tick = tokio::time::Instant::now();
                }
            }
        });

        Self { rx, tx }
    }

    pub async fn next(&mut self) -> Result<Event> {
        self.rx.recv()
            .await
            .ok_or_else(|| anyhow::anyhow!("Event channel closed"))
    }
}
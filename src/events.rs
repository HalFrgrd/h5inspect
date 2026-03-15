use std::time::Duration;

use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent};

use crate::app::NodeIdT;
use crate::tree::TreeNode;

#[derive(Clone, Debug)]
pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    AnimationTick,
    Resize,
    TreeUpdate(TreeNode<NodeIdT>),
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct EventHandler {
    pub receiver: tokio::sync::mpsc::UnboundedReceiver<Event>,
}

impl EventHandler {
    pub fn new(receiver: tokio::sync::mpsc::UnboundedReceiver<Event>) -> Self {
        Self { receiver }
    }

    pub fn next_event(&mut self) -> Event {
        if event::poll(Duration::from_millis(100)).unwrap() {
            match event::read().unwrap() {
                CrosstermEvent::Key(key) => Event::Key(key),
                CrosstermEvent::Mouse(mouse) => Event::Mouse(mouse),
                CrosstermEvent::Resize(_, _) => Event::Resize,
                _ => Event::AnimationTick, // Ignore other events for now
            }
        } else {
            while let Ok(ev) = self.receiver.try_recv() {
                if matches!(ev, Event::TreeUpdate(_)) {
                    return ev;
                }
            }
            Event::AnimationTick
        }
    }
}

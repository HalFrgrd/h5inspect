use std::time::Duration;

use crossterm::event::{Event as CrosstermEvent, KeyEvent, MouseEvent};
use futures::{FutureExt, StreamExt};

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
    pub sender: tokio::sync::mpsc::UnboundedSender<Event>,
    pub receiver: tokio::sync::mpsc::UnboundedReceiver<Event>,
    handler: tokio::task::JoinHandle<()>,
}

impl EventHandler {
    pub fn new() -> Self {
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        let sender_clone = sender.clone();
        let handler = tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            let mut animation_tick = tokio::time::interval(Duration::from_millis(100));
            loop {
                let tick_delay = animation_tick.tick();
                let crossterm_event = reader.next().fuse();
                tokio::select! {
                    _ = sender_clone.closed() => break,
                    _ = tick_delay => {
                        sender_clone.send(Event::AnimationTick).unwrap();
                    }
                    Some(Ok(evt)) = crossterm_event =>{
                        match evt {
                            CrosstermEvent::Key(key) => {
                                if key.kind == crossterm::event::KeyEventKind::Press {
                                  sender_clone.send(Event::Key(key)).unwrap();
                                }
                            }
                            CrosstermEvent::Mouse(mouse) => {
                                if mouse.kind != crossterm::event::MouseEventKind::Moved {
                                    // Ignore mouse move events to reduce event spam
                                    // drag events are still sent through
                                    sender_clone.send(Event::Mouse(mouse)).unwrap();
                                }
                            }
                            CrosstermEvent::Resize(_, _) => {
                                sender_clone.send(Event::Resize).unwrap();
                            }
                            CrosstermEvent::FocusLost => {}
                            CrosstermEvent::FocusGained => {}
                            CrosstermEvent::Paste(_) => {}
                        }
                    }
                }
            }
        });
        Self {
            sender,
            receiver,
            handler,
        }
    }
}

use crate::id_types::Ship;
use nalgebra::Point2;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;

pub enum SystemMessage {
    CommandRequest(CommandRequest),
    CommandResponse(Ship),
    AddComponent(Ship),
    UpdateComponent(Ship),
    Log(Ship),
    Delete(Ship),
}

pub enum CommandRequest {
    Heartbeat(Ship),
    GridCell(Point2<i16>),
}

pub struct System {
    channel: Receiver<SystemMessage>,
}

impl System {
    pub fn init() -> (JoinHandle<()>, Sender<SystemMessage>) {
        let (tx, rx) = mpsc::channel();

        let mut system = Self { channel: rx };

        let handle = thread::spawn(move || {
            system.update_loop();
        });

        (handle, tx)
    }

    fn update_loop(&mut self) {
        while let Ok(message) = self.channel.recv() {
            match message {
                SystemMessage::CommandRequest(req) => match req {
                    CommandRequest::Heartbeat(ship_id) => {}
                    CommandRequest::GridCell(grid_cell) => {}
                },
                SystemMessage::CommandResponse(ship_id) => {}
                SystemMessage::AddComponent(ship_id) => {}
                SystemMessage::UpdateComponent(ship_id) => {}
                SystemMessage::Log(ship_id) => {}
                SystemMessage::Delete(ship_id) => {}
            }
        }
    }
}

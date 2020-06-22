use crate::id_types::{Asteroid, Ship, User};
use crate::modules::samplers::Sampler;
use crate::modules::scanners::Scanner;
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
    GenerateResource(Asteroid, User, Scanner),
    ExtractResource(Asteroid, Ship, Sampler),
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
                    CommandRequest::Heartbeat(_ship_id) => {}
                    CommandRequest::GridCell(_grid_cell) => {}
                    CommandRequest::GenerateResource(_asteroid_id, _user_id, _scanner) => {}
                    CommandRequest::ExtractResource(_asteroid_id, _ship_id, _sampler) => {}
                },
                SystemMessage::CommandResponse(_ship_id) => {}
                SystemMessage::AddComponent(_ship_id) => {}
                SystemMessage::UpdateComponent(_ship_id) => {}
                SystemMessage::Log(_ship_id) => {}
                SystemMessage::Delete(_ship_id) => {}
            }
        }
    }
}

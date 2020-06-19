use crate::id_types::Ship;
use nalgebra::Point3;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;

pub enum SystemMessage {
    UpdatePosition(Ship, Point3<f64>),
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
        while let Ok(result) = self.channel.recv() {
            match result {
                SystemMessage::UpdatePosition(ship_id, position) => {}
            }
        }
    }
}

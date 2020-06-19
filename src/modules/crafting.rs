use crate::id_types::{DatabaseId, Module, Ship};
use crate::modules::module_inventory::ModuleResources;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;

pub enum SystemMessage {
    CraftModule(Ship),
}

pub struct System {
    channel: Receiver<SystemMessage>,
    //user ids
    //crafting levels
}

impl System {
    pub fn init(capacity: usize) -> (JoinHandle<()>, Sender<SystemMessage>) {
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
                SystemMessage::CraftModule(ship_id) => (),
            }
        }
    }

    fn craft_module(
        &mut self,
        user_id: DatabaseId,
        ship_id: Ship,
        module_id: Module,
        properties: [u8; 5],
    ) {
        //verify if user can craft this module with these properties -> local read

        //get resource requirements from lib

        //read cargo resources to verify requirements -> extern read

        //send message to add new module and remove resource used -> atomic?
    }
}

pub struct ModuleData {
    name: String,
    creator: DatabaseId,
    properties: [u8; 5],
    pub resources: ModuleResources,
}

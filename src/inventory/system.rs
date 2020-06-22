use crate::id_types::{Module, Resource, Ship};
use crate::inventory::modules::Modules;
use crate::inventory::resources::Resources;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;

pub enum SystemMessage {
    AddComponent(Ship, Inventory),
    RemoveComponent(Ship),
    UpdateModuleDurability(Ship, Module, i32),
    AddOrUpdateResource(Ship, Resource, i32),
    //CraftModule(Ship),
}

pub struct System {
    channel: Receiver<SystemMessage>,

    inventories: HashMap<Ship, Inventory>,
}

impl System {
    pub fn init(capacity: usize) -> (JoinHandle<()>, Sender<SystemMessage>) {
        let (tx, rx) = mpsc::channel();

        let mut system = Self {
            channel: rx,

            inventories: HashMap::with_capacity(capacity),
        };

        let handle = thread::spawn(move || {
            system.update_loop();
        });

        (handle, tx)
    }

    fn update_loop(&mut self) {
        while let Ok(message) = self.channel.recv() {
            match message {
                SystemMessage::AddComponent(ship_id, component) => {
                    self.inventories.insert(ship_id, component);
                }
                SystemMessage::RemoveComponent(ship_id) => {
                    self.inventories.remove(&ship_id);
                }
                SystemMessage::UpdateModuleDurability(ship_id, module_id, delta) => {
                    if let Some(inv) = self.inventories.get_mut(&ship_id) {
                        inv.modules.update_module_durability(&module_id, delta);
                    }

                    //TODO send update to spatialOS
                }
                SystemMessage::AddOrUpdateResource(ship_id, resource_id, quantity) => {
                    if let Some(inv) = self.inventories.get_mut(&ship_id) {
                        inv.resources.update_or_insert(&resource_id, quantity);
                    }

                    //TODO send update to spatialOS
                }
            }
        }
    }
}

pub struct Inventory /*Placeholder*/ {
    modules: Modules,
    resources: Resources,
}

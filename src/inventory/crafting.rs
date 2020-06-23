use crate::id_types::Ship;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, RwLock};
use std::thread;
use std::thread::JoinHandle;

pub enum SystemMessage {
    AddComponent(Ship, [u8; 5]),
    UpdateComponent(Ship, [u8; 5]),
    RemoveComponent(Ship),
}

pub struct System {
    channel: Receiver<SystemMessage>,

    levels: Arc<CraftingLevels>,
}

impl System {
    pub fn init(capacity: usize) -> (JoinHandle<()>, Sender<SystemMessage>, Arc<CraftingLevels>) {
        let (tx, rx) = mpsc::channel();

        let mut system = Self {
            channel: rx,

            levels: Arc::new(CraftingLevels::init(capacity)),
        };

        let arc = Arc::clone(&system.levels);

        let handle = thread::spawn(move || {
            system.update_loop();
        });

        (handle, tx, arc)
    }

    fn update_loop(&mut self) {
        while let Ok(result) = self.channel.recv() {
            match result {
                SystemMessage::AddComponent(ship_id, data) => self.levels.add(&ship_id, &data),
                SystemMessage::UpdateComponent(ship_id, data) => self.levels.add(&ship_id, &data),
                SystemMessage::RemoveComponent(ship_id) => self.levels.remove(&ship_id),
            }
        }
    }
}

pub struct CraftingLevels {
    data: RwLock<HashMap<Ship, [u8; 5]>>,
}

impl CraftingLevels {
    fn init(capacity: usize) -> Self {
        Self {
            data: RwLock::new(HashMap::with_capacity(capacity)),
        }
    }

    fn add(&self, ship_id: &Ship, data: &[u8; 5]) {
        if let Ok(mut hash_map) = self.data.write() {
            hash_map.insert(*ship_id, *data);
        }
    }

    fn remove(&self, ship_id: &Ship) {
        if let Ok(mut hash_map) = self.data.write() {
            hash_map.remove(ship_id);
        }
    }

    pub fn read(&self, ship_id: &Ship) -> Option<[u8; 5]> {
        let data = self.data.read();

        let data = match data {
            Ok(data) => data,
            Err(_) => return None,
        };

        match data.get(ship_id) {
            Some(data) => Some(*data),
            None => None,
        }
    }
}

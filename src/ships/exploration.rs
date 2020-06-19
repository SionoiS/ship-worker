use crate::id_types::Ship;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, RwLock};
use std::thread;
use std::thread::JoinHandle;

pub enum SystemMessage {
    AddComponent(Ship, ExplorationData),
    UpdateComponent(Ship, ExplorationData),
    RemoveComponent(Ship),
}

pub struct ExplorationData;

type ConcurrentHashMap = Arc<RwLock<HashMap<Ship, ExplorationData>>>;

pub struct System {
    channel: Receiver<SystemMessage>,

    ids: ConcurrentHashMap,
}

impl System {
    pub fn init(capacity: usize) -> (JoinHandle<()>, Sender<SystemMessage>, ConcurrentHashMap) {
        let (tx, rx) = mpsc::channel();

        let mut system = Self {
            channel: rx,

            ids: Arc::new(RwLock::new(HashMap::with_capacity(capacity))),
        };

        let arc = Arc::clone(&system.ids);

        let handle = thread::spawn(move || {
            system.update_loop();
        });

        (handle, tx, arc)
    }

    fn update_loop(&mut self) {
        while let Ok(result) = self.channel.recv() {
            match result {
                SystemMessage::AddComponent(id, data) => self.add_component(id, data),
                SystemMessage::UpdateComponent(id, data) => self.update_component(id, data),
                SystemMessage::RemoveComponent(id) => self.remove_component(id),
            }
        }
    }

    fn add_component(&mut self, id: Ship, data: ExplorationData) {
        if let Ok(mut hash_map) = self.ids.write() {
            hash_map.insert(id, data);
        }
    }

    fn update_component(&mut self, id: Ship, data: ExplorationData) {
        if let Ok(mut hash_map) = self.ids.write() {
            hash_map.insert(id, data);
        }
    }

    fn remove_component(&mut self, id: Ship) {
        if let Ok(mut hash_map) = self.ids.write() {
            hash_map.remove(&id);
        }
    }
}

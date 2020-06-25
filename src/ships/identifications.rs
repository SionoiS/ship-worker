use procedural_generation::id_types::{Ship, User};
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, RwLock};
use std::thread;
use std::thread::JoinHandle;

pub enum SystemMessage {
    AddComponent(Ship, User),
    UpdateComponent(Ship, User),
    RemoveComponent(Ship),
}

pub struct System {
    channel: Receiver<SystemMessage>,

    identifiers: Arc<Identifiers>,
}

impl System {
    pub fn init(capacity: usize) -> (JoinHandle<()>, Sender<SystemMessage>, Arc<Identifiers>) {
        let (tx, rx) = mpsc::channel();

        let mut system = Self {
            channel: rx,

            identifiers: Arc::new(Identifiers::init(capacity)),
        };

        let arc = Arc::clone(&system.identifiers);

        let handle = thread::spawn(move || {
            system.update_loop();
        });

        (handle, tx, arc)
    }

    fn update_loop(&mut self) {
        while let Ok(result) = self.channel.recv() {
            match result {
                SystemMessage::AddComponent(id, data) => self.identifiers.add(&id, &data),
                SystemMessage::UpdateComponent(id, data) => self.identifiers.add(&id, &data),
                SystemMessage::RemoveComponent(id) => self.identifiers.remove(&id),
            }
        }
    }
}

pub struct Identifiers {
    data: RwLock<HashMap<Ship, User>>,
}

impl Identifiers {
    fn init(capacity: usize) -> Self {
        Self {
            data: RwLock::new(HashMap::with_capacity(capacity)),
        }
    }

    fn add(&self, ship_id: &Ship, asteroid: &User) {
        if let Ok(mut hash_map) = self.data.write() {
            hash_map.insert(*ship_id, *asteroid);
        }
    }

    fn remove(&self, ship_id: &Ship) {
        if let Ok(mut hash_map) = self.data.write() {
            hash_map.remove(ship_id);
        }
    }

    pub fn read(&self, ship_id: &Ship) -> Option<User> {
        let user = self.data.read();

        let user = match user {
            Ok(user) => user,
            Err(_) => return None,
        };

        match user.get(ship_id) {
            Some(user) => Some(*user),
            None => None,
        }
    }
}

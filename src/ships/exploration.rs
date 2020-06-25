use procedural_generation::id_types::{Asteroid, Ship};
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, RwLock};
use std::thread;
use std::thread::JoinHandle;

pub enum SystemMessage {
    AddComponent(Ship, Asteroid),
    UpdateComponent(Ship, Asteroid),
    RemoveComponent(Ship),
}

pub struct System {
    channel: Receiver<SystemMessage>,

    asteroids: Arc<Asteroids>,
}

impl System {
    pub fn init(capacity: usize) -> (JoinHandle<()>, Sender<SystemMessage>, Arc<Asteroids>) {
        let (tx, rx) = mpsc::channel();

        let mut system = Self {
            channel: rx,

            asteroids: Arc::new(Asteroids::init(capacity)),
        };

        let arc = Arc::clone(&system.asteroids);

        let handle = thread::spawn(move || {
            system.update_loop();
        });

        (handle, tx, arc)
    }

    fn update_loop(&mut self) {
        while let Ok(result) = self.channel.recv() {
            match result {
                SystemMessage::AddComponent(id, data) => self.asteroids.add(&id, &data),
                SystemMessage::UpdateComponent(id, data) => self.asteroids.add(&id, &data),
                SystemMessage::RemoveComponent(id) => self.asteroids.remove(&id),
            }
        }
    }
}

pub struct Asteroids {
    data: RwLock<HashMap<Ship, Asteroid>>,
}

impl Asteroids {
    fn init(capacity: usize) -> Self {
        Self {
            data: RwLock::new(HashMap::with_capacity(capacity)),
        }
    }

    fn add(&self, ship_id: &Ship, asteroid: &Asteroid) {
        if let Ok(mut hash_map) = self.data.write() {
            hash_map.insert(*ship_id, *asteroid);
        }
    }

    fn remove(&self, ship_id: &Ship) {
        if let Ok(mut hash_map) = self.data.write() {
            hash_map.remove(ship_id);
        }
    }

    pub fn read(&self, ship_id: &Ship) -> Option<Asteroid> {
        let asteroid = self.data.read();

        let asteroid = match asteroid {
            Ok(asteroid) => asteroid,
            Err(_) => return None,
        };

        match asteroid.get(ship_id) {
            Some(asteroid) => Some(*asteroid),
            None => None,
        }
    }
}

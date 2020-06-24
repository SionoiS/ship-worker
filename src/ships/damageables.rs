use crate::id_types::Ship;
use crate::spatial_os::connexion::{SystemMessage as SpatialOSMsg, UpdateComponent};
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, RwLock};
use std::thread;
use std::thread::JoinHandle;

pub enum SystemMessage {
    AddComponent(Ship, NonZeroU32),
    RemoveComponent(Ship),

    TakeDamageCommand(Ship, NonZeroU32),
}

pub struct System {
    channel: Receiver<SystemMessage>,
    spatial_os: Sender<SpatialOSMsg>,

    healths: Arc<Healths>,
}

impl System {
    pub fn init(
        capacity: usize,
        spatial_os: Sender<SpatialOSMsg>,
    ) -> (JoinHandle<()>, Sender<SystemMessage>, Arc<Healths>) {
        let (tx, channel) = mpsc::channel();

        let mut system = Self {
            channel,
            spatial_os,

            healths: Arc::new(Healths::init(capacity)),
        };

        let arc = Arc::clone(&system.healths);

        let handle = thread::spawn(move || {
            system.update_loop();
        });

        (handle, tx, arc)
    }

    fn update_loop(&mut self) {
        while let Ok(result) = self.channel.recv() {
            match result {
                SystemMessage::AddComponent(id, data) => self.healths.add(&id, data),
                SystemMessage::RemoveComponent(id) => self.healths.remove(&id),
                SystemMessage::TakeDamageCommand(id, data) => self.take_damage(&id, data),
            }
        }
    }

    fn take_damage(&mut self, ship_id: &Ship, damage: NonZeroU32) {
        let mut hash_map = self.healths.data.write().expect("Lock poisoned");

        let hp = match hash_map.get_mut(ship_id) {
            Some(hp) => *hp,
            None => return,
        };

        if let Some(result) = hp.get().checked_sub(damage.get()) {
            if let Some(result) = NonZeroU32::new(result) {
                hash_map.insert(*ship_id, result);

                let message = SpatialOSMsg::UpdateComponent(
                    *ship_id,
                    UpdateComponent::Damageable(result.get()),
                );

                self.spatial_os
                    .send(message)
                    .expect("SpatialOS connexion terminated");

                return;
            }
        }

        hash_map.remove(ship_id);

        let message = SpatialOSMsg::Delete(*ship_id);

        self.spatial_os
            .send(message)
            .expect("SpatialOS connexion terminated");
    }
}

pub struct Healths {
    data: RwLock<HashMap<Ship, NonZeroU32>>,
}

impl Healths {
    fn init(capacity: usize) -> Self {
        Self {
            data: RwLock::new(HashMap::with_capacity(capacity)),
        }
    }

    fn add(&self, ship_id: &Ship, health: NonZeroU32) {
        let mut hash_map = self.data.write().expect("Lock poisoned");

        hash_map.insert(*ship_id, health);
    }

    fn remove(&self, ship_id: &Ship) {
        let mut hash_map = self.data.write().expect("Lock poisoned");

        hash_map.remove(ship_id);
    }

    pub fn read(&self, ship_id: &Ship) -> Option<NonZeroU32> {
        let hash_map = self.data.read().expect("lock poisoned");

        match hash_map.get(ship_id) {
            Some(health) => Some(*health),
            None => None,
        }
    }
}

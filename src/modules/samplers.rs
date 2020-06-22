use crate::id_types::{Module, Resource, Ship};
use crate::modules::cooldowns::SharedIds;
use crate::modules::cooldowns::SystemMessage as CooldownMsg;
use crate::ships::exploration::Asteroids;
use crate::spatial_os::connexion::{CommandRequest, SystemMessage as SpatialOSMsg};
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;

pub enum SystemMessage {
    AddComponent(Ship, Sampler),
    UpdateComponent(Ship, Sampler),
    RemoveComponent(Ship),

    CommandResponse(Ship, Resource, u32),

    UseScanner(Ship),
}

#[derive(Copy, Clone)]
pub struct Sampler /*Placeholder*/ {
    id: Module,
}

pub struct System {
    channel: Receiver<SystemMessage>,
    spatial_os: Sender<SpatialOSMsg>,
    cooldowns: Sender<CooldownMsg>,

    samplers: HashMap<Ship, Sampler>,

    asteroids: Arc<Asteroids>,
    modules: Arc<SharedIds>,
}

impl System {
    pub fn init(
        capacity: usize,
        spatial_os: Sender<SpatialOSMsg>,
        cooldowns: Sender<CooldownMsg>,
        asteroids: Arc<Asteroids>,
        modules: Arc<SharedIds>,
    ) -> (JoinHandle<()>, Sender<SystemMessage>) {
        let (tx, rx) = mpsc::channel();

        let mut system = Self {
            channel: rx,
            spatial_os,
            cooldowns,

            samplers: HashMap::with_capacity(capacity),
            asteroids,
            modules,
        };

        let handle = thread::spawn(move || {
            system.update_loop();
        });

        (handle, tx)
    }

    fn update_loop(&mut self) {
        while let Ok(result) = self.channel.recv() {
            match result {
                SystemMessage::AddComponent(id, data) => self.add_component(&id, &data),
                SystemMessage::UpdateComponent(id, data) => self.update_component(&id, &data),
                SystemMessage::RemoveComponent(id) => self.remove_component(&id),
                SystemMessage::CommandResponse(ship_id, resource_id, quantity) => {
                    self.process_response(&ship_id, &resource_id, quantity)
                }
                SystemMessage::UseScanner(id) => self.use_scanner(&id),
            }
        }
    }

    fn add_component(&mut self, id: &Ship, data: &Sampler) {
        self.samplers.insert(*id, *data);
    }

    fn update_component(&mut self, id: &Ship, data: &Sampler) {
        self.samplers.insert(*id, *data);
    }

    fn remove_component(&mut self, id: &Ship) {
        self.samplers.remove(&id);
    }

    fn use_scanner(&self, id: &Ship) {
        let sampler = self.samplers.get(id);
        let sampler = match sampler {
            Some(sampler) => *sampler,
            None => {
                return;
            }
        };

        if self.modules.on_cooldown(sampler.id) {
            return;
        }

        let asteroid = self.asteroids.read(id);
        let asteroid = match asteroid {
            Some(asteroid) => asteroid,
            None => {
                return;
            }
        };

        //TODO reduce durability of module

        let message = CooldownMsg::StartTimer(sampler.id);

        self.cooldowns
            .send(message)
            .expect("Cooldown system terminated");

        let message =
            SpatialOSMsg::CommandRequest(CommandRequest::ExtractResource(asteroid, *id, sampler));

        self.spatial_os
            .send(message)
            .expect("SpatialOS connexion terminated");
    }

    fn process_response(&self, ship_id: &Ship, resource_id: &Resource, quantity: u32) {
        //TODO if command success guard close

        //TODO add resource to inventory
    }
}

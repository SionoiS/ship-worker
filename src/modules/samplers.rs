use crate::id_types::{Module, Resource, Ship};
use crate::inventory::{Inventories, SystemMessage as InvMsg};
use crate::modules::cooldowns::{Cooldowns, SystemMessage as CooldownMsg};
use crate::ships::exploration::Asteroids;
use crate::spatial_os::connexion::{CommandRequest, SystemMessage as SpatialOSMsg};
use procedural_generation::modules::samplers::SamplerStats;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;

pub enum SystemMessage {
    AddComponent(Ship, Module),
    UpdateComponent(Ship, Module),
    RemoveComponent(Ship),

    CommandResponse(Ship, Resource, u32),

    UseScanner(Ship),
}

pub struct System {
    channel: Receiver<SystemMessage>,
    spatial_os: Sender<SpatialOSMsg>,
    cooldown: Sender<CooldownMsg>,
    inventory: Sender<InvMsg>,

    samplers: HashMap<Ship, Module>,

    asteroids: Arc<Asteroids>,
    cooldowns: Arc<Cooldowns>,
    inventories: Arc<Inventories>,
}

impl System {
    pub fn init(
        capacity: usize,
        spatial_os: Sender<SpatialOSMsg>,
        cooldown: Sender<CooldownMsg>,
        inventory: Sender<InvMsg>,
        asteroids: Arc<Asteroids>,
        cooldowns: Arc<Cooldowns>,
        inventories: Arc<Inventories>,
    ) -> (JoinHandle<()>, Sender<SystemMessage>) {
        let (tx, rx) = mpsc::channel();

        let mut system = Self {
            channel: rx,
            spatial_os,
            cooldown,
            inventory,

            samplers: HashMap::with_capacity(capacity),
            asteroids,
            cooldowns,
            inventories,
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

    fn add_component(&mut self, id: &Ship, data: &Module) {
        self.samplers.insert(*id, *data);
    }

    fn update_component(&mut self, id: &Ship, data: &Module) {
        self.samplers.insert(*id, *data);
    }

    fn remove_component(&mut self, id: &Ship) {
        self.samplers.remove(&id);
    }

    fn use_scanner(&self, ship_id: &Ship) {
        let sampler_id = self.samplers.get(ship_id);
        let sampler_id = match sampler_id {
            Some(sampler_id) => sampler_id,
            None => return,
        };

        if self.cooldowns.is_active(sampler_id) {
            return;
        }

        let props = self.inventories.get_module_properties(ship_id, sampler_id);
        let sampler = match props {
            Some(props) => SamplerStats::from_properties(&props),
            None => return,
        };

        let asteroid = self.asteroids.read(ship_id);
        let asteroid = match asteroid {
            Some(asteroid) => asteroid,
            None => return,
        };

        let message = CooldownMsg::StartTimer(*sampler_id);

        self.cooldown
            .send(message)
            .expect("Cooldown system terminated");

        let message = InvMsg::UpdateModuleDurability(*ship_id, *sampler_id, -1);

        self.inventory
            .send(message)
            .expect("Inventory system terminated");

        let message = SpatialOSMsg::CommandRequest(CommandRequest::ExtractResource(
            asteroid, *ship_id, sampler,
        ));

        self.spatial_os
            .send(message)
            .expect("SpatialOS connexion terminated");
    }

    fn process_response(&self, ship_id: &Ship, resource_id: &Resource, quantity: u32) {
        //TODO guard clause if command success

        let message = InvMsg::AddOrUpdateResource(*ship_id, *resource_id, quantity as i32);

        self.inventory
            .send(message)
            .expect("Inventory system terminated");
    }
}

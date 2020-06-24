use crate::id_types::{Module, Resource, Ship};
use crate::inventory::SystemMessage as InvMsg;
use crate::modules::cooldowns::SharedIds;
use crate::modules::cooldowns::SystemMessage as CooldownMsg;
use crate::ships::exploration::Asteroids;
use crate::ships::identifications::Identifiers;
use crate::spatial_os::connexion::{
    CommandRequest, SystemMessage as SpatialOSMsg, UpdateComponent,
};
use procedural_generation::modules::scanners::ScannerStats;
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
    cooldowns: Sender<CooldownMsg>,
    inventory: Sender<InvMsg>,

    scanners: HashMap<Ship, Module>,

    asteroids: Arc<Asteroids>,
    modules: Arc<SharedIds>,
    identifiers: Arc<Identifiers>,
}

impl System {
    pub fn init(
        capacity: usize,
        spatial_os: Sender<SpatialOSMsg>,
        cooldowns: Sender<CooldownMsg>,
        inventory: Sender<InvMsg>,
        asteroids: Arc<Asteroids>,
        modules: Arc<SharedIds>,
        identifiers: Arc<Identifiers>,
    ) -> (JoinHandle<()>, Sender<SystemMessage>) {
        let (tx, rx) = mpsc::channel();

        let mut system = Self {
            channel: rx,
            spatial_os,
            cooldowns,
            inventory,

            scanners: HashMap::with_capacity(capacity),
            asteroids,
            modules,
            identifiers,
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
        self.scanners.insert(*id, *data);
    }

    fn update_component(&mut self, id: &Ship, data: &Module) {
        self.scanners.insert(*id, *data);
    }

    fn remove_component(&mut self, id: &Ship) {
        self.scanners.remove(&id);
    }

    fn use_scanner(&self, ship_id: &Ship) {
        let scanner_id = self.scanners.get(ship_id);
        let scanner_id = match scanner_id {
            Some(scanner_id) => scanner_id,
            None => {
                return;
            }
        };

        if self.modules.on_cooldown(scanner_id) {
            return;
        }

        //TODO
        let props = self.inventory.read(scanner_id);
        let scanner = match props {
            Some(props) => ScannerStats::from_properties(props),
            None => {
                return;
            }
        };

        let asteroid = self.asteroids.read(ship_id);
        let asteroid = match asteroid {
            Some(asteroid) => asteroid,
            None => {
                return;
            }
        };

        let user = self.identifiers.read(ship_id);
        let user = match user {
            Some(user) => user,
            None => {
                return;
            }
        };

        let message = InvMsg::UpdateModuleDurability(*ship_id, *scanner_id, -1);

        self.inventory
            .send(message)
            .expect("Inventory system terminated");

        let message = CooldownMsg::StartTimer(*scanner_id);

        self.cooldowns
            .send(message)
            .expect("Cooldown system terminated");

        let message =
            SpatialOSMsg::CommandRequest(CommandRequest::GenerateResource(asteroid, user, scanner));

        self.spatial_os
            .send(message)
            .expect("SpatialOS connexion terminated");
    }

    fn process_response(&self, ship_id: &Ship, resource_id: &Resource, quantity: u32) {
        //TODO guard clause if command success

        let message = SpatialOSMsg::UpdateComponent(
            *ship_id,
            UpdateComponent::Scanner(*resource_id, quantity),
        );

        self.spatial_os
            .send(message)
            .expect("SpatialOs connexion terminated");
    }
}

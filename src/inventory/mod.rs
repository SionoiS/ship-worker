mod modules;
mod resources;

use crate::inventory::modules::{ModuleResources, ModuleStats, Modules};
use crate::inventory::resources::Resources;
use crate::ships::identifications::Identifiers;
use crate::spatial_os::connexion::{SystemMessage as SpatialOSMsg, UpdateComponent};
use procedural_generation::id_types::{Module, Resource, Ship, User};
use procedural_generation::modules::samplers::SamplerStats;
use procedural_generation::modules::scanners::ScannerStats;
use procedural_generation::modules::sensors::SensorStats;
use std::collections::HashMap;
use std::mem;
use std::num::NonZeroU32;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;
use std::thread::JoinHandle;

pub enum SystemMessage {
    AddOrUpdateComponent(Ship, Inventory),
    RemoveComponent(Ship),
    UpdateModuleDurability(Ship, Module, i32),
    AddOrUpdateResource(Ship, Resource, i32),
    CraftModule(Ship, Module, String, Vec<u8>, Vec<Resource>),
}

pub struct System {
    channel: Receiver<SystemMessage>,
    spatial_os: Sender<SpatialOSMsg>,

    inventories: Arc<Inventories>,

    identifiers: Arc<Identifiers>,
}

impl System {
    pub fn init(
        capacity: usize,
        spatial_os: Sender<SpatialOSMsg>,
        identifiers: Arc<Identifiers>,
    ) -> (JoinHandle<()>, Sender<SystemMessage>, Arc<Inventories>) {
        let (tx, channel) = mpsc::channel();

        let mut system = Self {
            channel,
            spatial_os,

            inventories: Arc::new(Inventories::new(capacity)),

            identifiers,
        };

        let arc = Arc::clone(&system.inventories);

        let handle = thread::spawn(move || {
            system.update_loop();
        });

        (handle, tx, arc)
    }

    fn update_loop(&mut self) {
        while let Ok(message) = self.channel.recv() {
            match message {
                SystemMessage::AddOrUpdateComponent(ship_id, inventory) => {
                    self.inventories.add(&ship_id, inventory)
                }
                SystemMessage::RemoveComponent(ship_id) => self.inventories.remove(&ship_id),
                SystemMessage::UpdateModuleDurability(ship_id, module_id, delta) => {
                    self.update_module_durability(&ship_id, &module_id, delta)
                }
                SystemMessage::AddOrUpdateResource(ship_id, resource_id, quantity) => {
                    self.add_or_update_resource(&ship_id, &resource_id, quantity)
                }
                SystemMessage::CraftModule(ship_id, module_id, name, craft_levels, resources) => {
                    self.craft_module(&ship_id, &module_id, name, &craft_levels, &resources)
                }
            }
        }
    }

    fn update_module_durability(&mut self, ship_id: &Ship, module_id: &Module, delta: i32) {
        let mut inv = self.inventories.data.write().expect("Lock poisoned");

        let inv = inv.get_mut(&ship_id);
        let inv = match inv {
            Some(inv) => inv,
            None => return,
        };

        inv.modules.update_module_durability(module_id, delta);

        let message =
            SpatialOSMsg::UpdateComponent(*ship_id, UpdateComponent::Inventory(inv.clone()));

        self.spatial_os
            .send(message)
            .expect("SpatialOS connexion terminated");
    }

    fn add_or_update_resource(&mut self, ship_id: &Ship, resource_id: &Resource, quantity: i32) {
        let mut inv = self.inventories.data.write().expect("Lock poisoned");

        let inv = inv.get_mut(&ship_id);
        let inv = match inv {
            Some(inv) => inv,
            None => return,
        };

        inv.resources.update_or_insert(&resource_id, quantity);

        let message =
            SpatialOSMsg::UpdateComponent(*ship_id, UpdateComponent::Inventory(inv.clone()));

        self.spatial_os
            .send(message)
            .expect("SpatialOS connexion terminated");
    }

    fn craft_module(
        &mut self,
        ship_id: &Ship,
        module_id: &Module,
        name: String,
        craft_levels: &[u8],
        resources: &[Resource],
    ) {
        let mut inv = self.inventories.data.write().expect("Lock poisoned");

        let inv = inv.get_mut(ship_id);
        let inv = match inv {
            Some(inv) => inv,
            None => return,
        };

        let creator = self.identifiers.read(ship_id);
        let creator = match creator {
            Some(creator) => creator,
            None => return,
        };

        let requirements = match module_id {
            Module::Sampler(_) => {
                if let Ok(stats) = SamplerStats::from_properties(craft_levels) {
                    stats.get_requirements()
                } else {
                    return;
                }
            }
            Module::Scanner(_) => {
                if let Ok(stats) = ScannerStats::from_properties(craft_levels) {
                    stats.get_requirements()
                } else {
                    return;
                }
            }
            Module::Sensor(_) => {
                if let Ok(stats) = SensorStats::from_properties(craft_levels) {
                    stats.get_requirements()
                } else {
                    return;
                }
            }
        };

        if requirements.len() != resources.len() {
            return;
        }

        for (i, requirement) in requirements.iter().enumerate() {
            let (req_resource, req_quantity) = requirement;

            //discriminant is used to compare enum variants, disregarding struct values
            if mem::discriminant(req_resource) != mem::discriminant(&resources[i]) {
                return;
            }

            if !inv.resources.has_enough(&resources[i], *req_quantity) {
                return;
            }
        }

        let quantities = requirements
            .into_iter()
            .map(|tuple| tuple.1)
            .collect::<Vec<NonZeroU32>>();

        inv.craft_new_module(
            module_id,
            name,
            &creator,
            craft_levels,
            resources,
            &quantities,
        );

        let message =
            SpatialOSMsg::UpdateComponent(*ship_id, UpdateComponent::Inventory(inv.clone()));

        self.spatial_os
            .send(message)
            .expect("SpatialOS connexion terminated");
    }
}

pub struct Inventories {
    data: RwLock<HashMap<Ship, Inventory>>,
}

impl Inventories {
    fn new(capacity: usize) -> Self {
        Self {
            data: RwLock::new(HashMap::with_capacity(capacity)),
        }
    }

    fn add(&self, ship_id: &Ship, inventory: Inventory) {
        if let Ok(mut hash_map) = self.data.write() {
            hash_map.insert(*ship_id, inventory);
        }
    }

    fn remove(&self, ship_id: &Ship) {
        if let Ok(mut hash_map) = self.data.write() {
            hash_map.remove(ship_id);
        }
    }

    pub fn get_module_properties(&self, ship_id: &Ship, module_id: &Module) -> Option<Vec<u8>> {
        let hash_map = self.data.read().expect("Lock poisoned");

        let inv = match hash_map.get(ship_id) {
            Some(inv) => inv,
            None => return None,
        };

        match inv.modules.modules.get(module_id) {
            Some(module) => Some(module.get_properties()),
            None => None,
        }
    }
}

#[derive(Clone)]
pub struct Inventory /*Placeholder Component*/ {
    modules: Modules,
    resources: Resources,
}

impl Inventory {
    fn new(capacity: usize) -> Self {
        Self {
            modules: Modules::with_capacity(capacity),
            resources: Resources::with_capacity(capacity),
        }
    }

    fn craft_new_module(
        &mut self,
        module_id: &Module,
        name: String,
        creator: &User,
        properties: &[u8],
        resources: &[Resource],
        quantities: &[NonZeroU32],
    ) {
        let module_res = ModuleResources::new(resources, quantities);

        let module = ModuleStats::new(name, *creator, properties, module_res);

        self.modules.add(module_id, module);

        for (i, resource) in resources.iter().enumerate() {
            self.resources
                .update_or_insert(resource, -(quantities[i].get() as i32));
        }
    }
}

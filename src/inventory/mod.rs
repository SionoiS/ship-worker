pub mod crafting;
mod modules;
mod resources;

use crate::id_types::{DatabaseId, Module, Resource, Ship, User};
use crate::inventory::crafting::CraftingLevels;
use crate::inventory::modules::{ModuleData, ModuleResources, Modules};
use crate::inventory::resources::Resources;
use crate::ships::identifications::Identifiers;
use crate::spatial_os::connexion::{SystemMessage as SpatialOSMsg, UpdateComponent};
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;

pub enum SystemMessage {
    AddOrUpdateComponent(Ship, Inventory),
    RemoveComponent(Ship),
    UpdateModuleDurability(Ship, Module, i32),
    AddOrUpdateResource(Ship, Resource, i32),
    CraftModule(Ship, Module, String, [u8; 5], Vec<Resource>),
}

pub struct System {
    channel: Receiver<SystemMessage>,
    spatial_os: Sender<SpatialOSMsg>,

    inventories: HashMap<Ship, Inventory>,

    levels: Arc<CraftingLevels>,
    identifiers: Arc<Identifiers>,
}

impl System {
    pub fn init(
        capacity: usize,
        spatial_os: Sender<SpatialOSMsg>,
        levels: Arc<CraftingLevels>,
        identifiers: Arc<Identifiers>,
    ) -> (JoinHandle<()>, Sender<SystemMessage>) {
        let (tx, channel) = mpsc::channel();

        let mut system = Self {
            channel,
            spatial_os,

            inventories: HashMap::with_capacity(capacity),

            levels,
            identifiers,
        };

        let handle = thread::spawn(move || {
            system.update_loop();
        });

        (handle, tx)
    }

    fn update_loop(&mut self) {
        while let Ok(message) = self.channel.recv() {
            match message {
                SystemMessage::AddOrUpdateComponent(ship_id, component) => {
                    self.inventories.insert(ship_id, component);
                }
                SystemMessage::RemoveComponent(ship_id) => {
                    self.inventories.remove(&ship_id);
                }
                SystemMessage::UpdateModuleDurability(ship_id, module_id, delta) => {
                    if let Some(inv) = self.inventories.get_mut(&ship_id) {
                        inv.modules.update_module_durability(&module_id, delta);

                        let message = SpatialOSMsg::UpdateComponent(
                            ship_id,
                            UpdateComponent::Inventory(inv.clone()),
                        );

                        self.spatial_os.send(message);
                    }
                }
                SystemMessage::AddOrUpdateResource(ship_id, resource_id, quantity) => {
                    if let Some(inv) = self.inventories.get_mut(&ship_id) {
                        inv.resources.update_or_insert(&resource_id, quantity);

                        let message = SpatialOSMsg::UpdateComponent(
                            ship_id,
                            UpdateComponent::Inventory(inv.clone()),
                        );

                        self.spatial_os.send(message);
                    }
                }
                SystemMessage::CraftModule(ship_id, module_id, name, craft_levels, resources) => {
                    self.try_craft_module(&ship_id, &module_id, name, &craft_levels, &resources)
                }
            }
        }
    }

    fn try_craft_module(
        &mut self,
        ship_id: &Ship,
        module_id: &Module,
        name: String,
        craft_levels: &[u8; 5],
        resources: &[Resource],
    ) {
        if let Some(inv) = self.inventories.get_mut(ship_id) {
            let creator = self.identifiers.read(ship_id);
            let creator = match creator {
                Some(creator) => creator,
                None => return,
            };

            let crafting_power = self.levels.read(ship_id);
            let crafting_power = match crafting_power {
                Some(crafting_power) => crafting_power,
                None => return,
            };

            for (i, craft_level) in craft_levels.iter().enumerate() {
                if *craft_level > crafting_power[i] {
                    return;
                }
            }

            //TODO get resource requirements for this module
            let (req_resources, req_quantities) =
                procedural_generation::modules::samplers::get_requirements();

            //TODO verify that resource from user match requirements
            let mut mask = 0b_00000000;
            for resource in resources {
                match resource {
                    Resource::Metal(_) => mask |= 0b_10000000,
                    Resource::Crystal(_) => mask |= 0b_01000000,
                    Resource::Radioactive(_) => mask |= 0b_00100000,
                    Resource::Organic(_) => mask |= 0b_00010000,
                }
            }

            if req_resources ^ mask != 0 {
                return;
            }

            //TODO verify quantity requirements
            for resource in resources {
                if !inv
                    .resources
                    .has_enough(resource, NonZeroU32::new(1).unwrap())
                {
                    return;
                }
            }

            inv.craft_new_module(
                module_id,
                name,
                creator,
                *craft_levels,
                resources,
                &req_quantities,
            );

            let message =
                SpatialOSMsg::UpdateComponent(*ship_id, UpdateComponent::Inventory(inv.clone()));

            self.spatial_os
                .send(message)
                .expect("SpatialOS connexion terminated");
        }
    }
}

#[derive(Clone)]
pub struct Inventory /*Placeholder Component*/ {
    modules: Modules,
    resources: Resources,
}

impl Inventory {
    fn new() -> Self {
        Self {
            modules: Modules::with_capacity(10),
            resources: Resources::with_capacity(10),
        }
    }

    fn craft_new_module(
        &mut self,
        module_id: &Module,
        name: String,
        creator: User,
        properties: [u8; 5],
        resources: &[Resource],
        quantities: &[NonZeroU32],
    ) {
        let module_res = ModuleResources::new(resources, quantities);

        let module = ModuleData::new(name, creator, properties, module_res);

        self.modules.add(*module_id, module);

        for (i, resource) in resources.iter().enumerate() {
            self.resources
                .update_or_insert(resource, -(quantities[i].get() as i32));
        }
    }
}

use crate::inventory::{Inventories, SystemMessage as InvMsg};
use crate::modules::cooldowns::{Cooldowns, SystemMessage as CooldownMsg};
use crate::ships::positions::Positions;
use crate::spatial_os::connexion::{SystemMessage as SpatialMsg, UpdateComponent};
use nalgebra::Vector3;
use procedural_generation::id_types::{Module, Ship};
use procedural_generation::modules::sensors::SensorStats;
use procedural_generation::resources::quantity::get_tier;
use procedural_generation::resources::rarity::get_samples;
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::{SystemTime, UNIX_EPOCH};

pub enum SystemMessage {
    AddComponent(Ship, Module),
    UpdateComponent(Ship, Module),
    RemoveComponent(Ship),

    UseSensor(Ship),
}

pub struct System {
    channel: Receiver<SystemMessage>,
    spatial_os: Sender<SpatialMsg>,
    cooldown: Sender<CooldownMsg>,
    inventory: Sender<InvMsg>,

    sensors: HashMap<Ship, Module>,

    positions: Arc<Positions>,
    cooldowns: Arc<Cooldowns>,
    inventories: Arc<Inventories>,
}

impl System {
    pub fn init(
        capacity: usize,
        spatial_os: Sender<SpatialMsg>,
        cooldown: Sender<CooldownMsg>,
        inventory: Sender<InvMsg>,
        positions: Arc<Positions>,
        cooldowns: Arc<Cooldowns>,
        inventories: Arc<Inventories>,
    ) -> (JoinHandle<()>, Sender<SystemMessage>) {
        let (tx, rx) = mpsc::channel();

        let mut system = Self {
            channel: rx,
            spatial_os,
            cooldown,
            inventory,

            sensors: HashMap::with_capacity(capacity),

            positions,
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
                SystemMessage::UseSensor(id) => self.use_sensor(&id),
            }
        }
    }

    fn add_component(&mut self, ship_id: &Ship, module_id: &Module) {
        self.sensors.insert(*ship_id, *module_id);
    }

    fn update_component(&mut self, ship_id: &Ship, module_id: &Module) {
        self.sensors.insert(*ship_id, *module_id);
    }

    fn remove_component(&mut self, ship_id: &Ship) {
        self.sensors.remove(&ship_id);
    }

    fn use_sensor(&self, ship_id: &Ship) {
        let sensor_id = self.sensors.get(ship_id);
        let sensor_id = match sensor_id {
            Some(sensor_id) => sensor_id,
            None => return,
        };

        if self.cooldowns.is_active(sensor_id) {
            return;
        }

        let props = self.inventories.get_module_properties(ship_id, sensor_id);
        let sensor = match props {
            Some(props) => match SensorStats::from_properties(&props) {
                Ok(sampler) => sampler,
                Err(_) => return,
            },
            None => return,
        };

        let position = self.positions.read(ship_id);
        let position = match position {
            Some(position) => position,
            None => return,
        };

        let message = CooldownMsg::StartTimer(*sensor_id);

        self.cooldown
            .send(message)
            .expect("Cooldown system terminated");

        let message = InvMsg::UpdateModuleDurability(*ship_id, *sensor_id, -1);

        self.inventory
            .send(message)
            .expect("Inventory system terminated");

        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("SystemTime before UNIX EPOCH!")
            .as_secs();

        let samples: Vec<u8> = sphere_points::calculate_coordinates(
            sensor.get_radius_range(),
            sensor.get_radius_resolution(),
            sensor.get_longitude_range(),
            sensor.get_longitude_resolution(),
            sensor.get_latitude_range(),
            sensor.get_latitude_resolution(),
        )
        .into_par_iter()
        .map(|point| {
            let sample = get_samples(
                &(position + point),
                time,
                &Vector3::new(0.0, 0.0, 0.0), //TODO get the correct data
                &Vector3::new(0.0, 0.0, 0.0),
                &Vector3::new(0.0, 0.0, 0.0),
                &Vector3::new(0, 0, 0),
                &[0u8; 512],
            );

            get_tier(sample)
        })
        .collect();

        let message = SpatialMsg::UpdateComponent(*ship_id, UpdateComponent::Sensor(samples));

        self.spatial_os
            .send(message)
            .expect("SpatialOS connexion terminated");
    }
}

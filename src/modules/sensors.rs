use crate::id_types::{Module, Ship};
use crate::inventory::SystemMessage as InvMsg;
use crate::modules::cooldowns::SharedIds;
use crate::modules::cooldowns::SystemMessage as CooldownMsg;
use crate::ships::positions::Positions;
use crate::spatial_os::connexion::{SystemMessage as SpatialMsg, UpdateComponent};
use nalgebra::Vector3;
use procedural_generation::resources::quantity::get_tier;
use procedural_generation::resources::rarity::get_samples;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::{SystemTime, UNIX_EPOCH};

pub enum SystemMessage {
    AddComponent(Ship, Sensor),
    UpdateComponent(Ship, Sensor),
    RemoveComponent(Ship),

    UseSensor(Ship),
}

#[derive(Copy, Clone)]
pub struct Sensor /*Placeholder Component*/ {
    id: Module,

    radius_range: f64,
    radius_resolution: i32,
    longitude_range: f64,
    longitude_resolution: i32,
    latitude_range: f64,
    latitude_resolution: i32,
}

pub struct System {
    channel: Receiver<SystemMessage>,
    spatial_os: Sender<SpatialMsg>,
    cooldowns: Sender<CooldownMsg>,
    inventory: Sender<InvMsg>,

    sensors: HashMap<Ship, Sensor>,
    positions: Arc<Positions>,
    modules: Arc<SharedIds>,
}

impl System {
    pub fn init(
        capacity: usize,
        spatial_os: Sender<SpatialMsg>,
        cooldowns: Sender<CooldownMsg>,
        inventory: Sender<InvMsg>,
        positions: Arc<Positions>,
        modules: Arc<SharedIds>,
    ) -> (JoinHandle<()>, Sender<SystemMessage>) {
        let (tx, rx) = mpsc::channel();

        let mut system = Self {
            channel: rx,
            spatial_os,
            cooldowns,
            inventory,

            sensors: HashMap::with_capacity(capacity),
            positions,
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
                SystemMessage::UseSensor(id) => self.use_sensor(&id),
            }
        }
    }

    fn add_component(&mut self, id: &Ship, data: &Sensor) {
        self.sensors.insert(*id, *data);
    }

    fn update_component(&mut self, id: &Ship, data: &Sensor) {
        self.sensors.insert(*id, *data);
    }

    fn remove_component(&mut self, id: &Ship) {
        self.sensors.remove(&id);
    }

    fn use_sensor(&self, ship_id: &Ship) {
        let sensor = self.sensors.get(ship_id);
        let sensor = match sensor {
            Some(sensor) => sensor,
            None => {
                return;
            }
        };

        let position = self.positions.read(ship_id);
        let position = match position {
            Some(position) => position,
            None => {
                return;
            }
        };

        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("SystemTime before UNIX EPOCH!")
            .as_secs();

        if self.modules.on_cooldown(sensor.id) {
            return;
        }

        let message = CooldownMsg::StartTimer(sensor.id);

        self.cooldowns
            .send(message)
            .expect("Cooldown system terminated");

        let message = InvMsg::UpdateModuleDurability(*ship_id, sensor.id, -1);

        self.inventory
            .send(message)
            .expect("Inventory system terminated");

        //TODO add survey to the map inventory

        let samples: Vec<u8> = sphere_points::calculate_coordinates(
            sensor.radius_range,
            sensor.radius_resolution,
            sensor.longitude_range,
            sensor.longitude_resolution,
            sensor.latitude_range,
            sensor.latitude_resolution,
        )
        .into_iter()
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

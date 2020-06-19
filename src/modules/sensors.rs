use crate::id_types::{Module, Ship};
use crate::modules::cooldowns::SharedIds;
use crate::modules::cooldowns::SystemMessage as CooldownMsg;
use crate::ships::positions::Positions;
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
pub struct Sensor {
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
    cooldowns: Sender<CooldownMsg>,

    sensors: HashMap<Ship, Sensor>,
    positions: Arc<Positions>,
    modules: Arc<SharedIds>,
}

impl System {
    pub fn init(
        capacity: usize,
        positions: Arc<Positions>,
        modules: Arc<SharedIds>,
        cooldowns: Sender<CooldownMsg>,
    ) -> (JoinHandle<()>, Sender<SystemMessage>) {
        let (tx, rx) = mpsc::channel();

        let mut system = Self {
            channel: rx,
            cooldowns,

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

    fn use_sensor(&self, id: &Ship) {
        let sensor = self.sensors.get(id);
        let sensor = match sensor {
            Some(sensor) => sensor,
            None => {
                return;
            }
        };

        let position = self.positions.read(id);
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

        //TODO reduce durability of module

        //TODO add survey to the map inventory

        let _samples: Vec<u8> = sphere_points::calculate_coordinates(
            sensor.radius_range,
            sensor.radius_resolution,
            sensor.longitude_range,
            sensor.longitude_resolution,
            sensor.latitude_range,
            sensor.latitude_resolution,
        )
        .into_iter()
        .map(|point| {
            let coords = position + point;
            let sample = get_samples(
                coords,
                time,
                Vector3::new(0.0, 0.0, 0.0), //TODO get the correct data
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, 0.0, 0.0),
                Vector3::new(0, 0, 0),
                [0u8; 512],
            );

            get_tier(sample)
        })
        .collect();

        //TODO send spatialOS event with rarity data
    }
}

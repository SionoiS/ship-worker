use crate::id_types::Ship;
use crate::spatial_os::connexion::{CommandRequest, SystemMessage as Message};
use nalgebra::{Point2, Point3};
use procedural_generation::world::asteroids::grid_cell_from_position;
use std::collections::{HashMap, HashSet};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, RwLock};
use std::thread;
use std::thread::JoinHandle;
use std::time::Instant;

pub enum SystemMessage {
    AddComponent(Ship, Point3<f64>),
    UpdateComponent(Ship, Point3<f64>),
    RemoveComponent(Ship),
    PositionIntervalUpdate(u32),
}

type ConcurrentHashMap = Arc<RwLock<HashMap<Ship, Point3<f64>>>>;

pub struct System {
    channel: Receiver<SystemMessage>,
    connexion: Sender<Message>,

    interval: u32,

    positions: ConcurrentHashMap,
    grid_cells: HashSet<Point2<i16>>,

    last_update: HashMap<Ship, Instant>,
}

impl System {
    pub fn init(
        capacity: usize,
        interval: u32,
        connexion: Sender<Message>,
    ) -> (JoinHandle<()>, Sender<SystemMessage>, ConcurrentHashMap) {
        let (tx, rx) = mpsc::channel();

        let mut system = Self {
            channel: rx,
            connexion,

            interval,

            positions: Arc::new(RwLock::new(HashMap::with_capacity(capacity))),
            grid_cells: HashSet::with_capacity(capacity),

            last_update: HashMap::with_capacity(capacity),
        };

        let arc = Arc::clone(&system.positions);

        let handle = thread::spawn(move || {
            system.update_loop();
        });

        (handle, tx, arc)
    }

    fn update_loop(&mut self) {
        while let Ok(result) = self.channel.recv() {
            match result {
                SystemMessage::AddComponent(ship_id, position) => {
                    self.add_component(ship_id, position)
                }
                SystemMessage::UpdateComponent(ship_id, position) => {
                    self.update_component(ship_id, position)
                }
                SystemMessage::RemoveComponent(ship_id) => self.remove_component(ship_id),
                SystemMessage::PositionIntervalUpdate(new_intv) => self.interval = new_intv,
            }
        }
    }

    fn add_component(&mut self, ship_id: Ship, position: Point3<f64>) {
        if let Ok(mut hash_map) = self.positions.write() {
            hash_map.insert(ship_id, position);
        }

        self.update_grid_cells(position);

        self.last_update.insert(ship_id, Instant::now());
    }

    fn update_component(&mut self, ship_id: Ship, position: Point3<f64>) {
        if let Ok(mut hash_map) = self.positions.write() {
            hash_map.insert(ship_id, position);
        }

        let now = Instant::now();
        let before = self.last_update.get(&ship_id);
        // Not sure what to do,
        // this case should never happen,
        // it would mean position, last_update and grid_cell desync.
        let before = match before {
            Some(before) => *before,
            None => return,
        };

        if now.duration_since(before).as_millis() > self.interval as u128 {
            //TODO send pos update to DB
        }

        self.update_grid_cells(position);
    }

    fn remove_component(&mut self, ship_id: Ship) {
        if let Ok(mut hash_map) = self.positions.write() {
            hash_map.remove(&ship_id);
        }

        self.last_update.remove(&ship_id);
    }

    fn update_grid_cells(&mut self, position: Point3<f64>) {
        let grid_cell = grid_cell_from_position(position);

        if self.grid_cells.insert(grid_cell) {
            let message = Message::CommandRequest(CommandRequest::GridCell(grid_cell));

            self.connexion.send(message);
        }
    }
}

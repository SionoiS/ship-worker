use crate::spatial_os::connexion::{CommandRequest, SystemMessage as Message};
use procedural_generation::id_types::Ship;
use rand::Rng;
use rand::SeedableRng;
use rand_xoshiro::Xoshiro128StarStar;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

pub enum SystemMessage {
    AddComponent(Ship),
    RemoveComponent(Ship),
    HeartbeatResponse(Ship),
    HeartbeatIntervalUpdate(u16),
}

pub struct System {
    channel: Receiver<SystemMessage>,
    connexion: Sender<Message>,

    prng: Xoshiro128StarStar,

    heartbeat_interval: u16,
    frame_time: Duration,

    entities: Vec<Ship>,
    entities_missed_hearbeat: HashMap<Ship, u8>,
}

impl System {
    pub fn init(
        capacity: usize,
        interval: u16,
        connexion: Sender<Message>,
    ) -> (JoinHandle<()>, Sender<SystemMessage>) {
        let (tx, rx) = mpsc::channel();

        let mut system = Self {
            channel: rx,
            connexion,

            prng: Xoshiro128StarStar::from_entropy(),

            heartbeat_interval: interval,
            frame_time: Duration::from_millis(interval as u64),

            entities: Vec::with_capacity(capacity),
            entities_missed_hearbeat: HashMap::with_capacity(capacity),
        };

        let handle = thread::spawn(move || {
            //GameLoop
            loop {
                let before_frame = Instant::now();

                system.update();

                let after_frame = Instant::now();

                let frame_duration = after_frame.duration_since(before_frame);

                if let Some(delta_time) = system.frame_time.checked_sub(frame_duration) {
                    std::thread::sleep(delta_time);
                } else {
                    println!(
                        "The frame took {:?}, too long couldn't sleep",
                        frame_duration
                    );
                }
            }
        });

        (handle, tx)
    }

    fn update(&mut self) {
        while let Ok(result) = self.channel.try_recv() {
            match result {
                SystemMessage::AddComponent(ship_id) => self.add_component(ship_id),
                SystemMessage::RemoveComponent(ship_id) => self.remove_component(ship_id),
                SystemMessage::HeartbeatResponse(ship_id) => self.process_response(ship_id),
                SystemMessage::HeartbeatIntervalUpdate(new_intv) => self.interval_update(new_intv),
            }
        }

        self.periodic_hearbeat();
    }

    fn add_component(&mut self, ship_id: Ship) {
        self.entities.push(ship_id);

        self.calculate_frame_rate();
    }

    fn remove_component(&mut self, ship_id: Ship) {
        for i in 0..self.entities.len() {
            if ship_id == self.entities[i] {
                self.entities.swap_remove(i);
                break;
            }
        }

        self.entities_missed_hearbeat.remove(&ship_id);

        self.calculate_frame_rate();
    }

    fn process_response(&mut self, ship_id: Ship /*, response: ResponseOP*/) {
        //TODO if response != success

        if let Some(missed_heartbeat) = self.entities_missed_hearbeat.get_mut(&ship_id) {
            *missed_heartbeat += 1;

            if *missed_heartbeat > 5 {
                let message = Message::Delete(ship_id);

                self.connexion
                    .send(message)
                    .expect("SpatialOS connexion terminated");
            }
        } else {
            self.entities_missed_hearbeat.insert(ship_id, 1);
        }
    }

    fn interval_update(&mut self, new_interval: u16) {
        self.heartbeat_interval = new_interval;

        self.calculate_frame_rate();
    }

    fn periodic_hearbeat(&mut self) {
        let count = self.entities.len();

        if count < 1 {
            return;
        }

        let ship_id = self.entities[self.prng.gen_range(0, count)];

        let message = Message::CommandRequest(CommandRequest::Heartbeat(ship_id));

        self.connexion
            .send(message)
            .expect("SpatialOS connexion terminated");
    }

    fn calculate_frame_rate(&mut self) {
        let count = self.entities_missed_hearbeat.len();

        if count > 0 {
            self.frame_time =
                Duration::from_millis((self.heartbeat_interval / count as u16) as u64);
        }
    }
}

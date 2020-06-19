use crate::id_types::Module;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, RwLock};
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

pub enum SystemMessage {
    AddComponent(Module, Duration),
    UpdateComponent(Module, Duration),
    RemoveComponent(Module),

    StartTimer(Module),
}

pub struct System {
    channel: Receiver<SystemMessage>,

    cold_data: ModuleCooldowns,
    hot_data: ActiveCooldowns,
}

impl System {
    pub fn init(capacity: usize) -> (JoinHandle<()>, Sender<SystemMessage>, Arc<SharedIds>) {
        let (tx, rx) = mpsc::channel();

        let mut system = Self {
            channel: rx,

            cold_data: ModuleCooldowns::new(capacity),
            hot_data: ActiveCooldowns::new(capacity / 10),
        };

        let arc = Arc::clone(&system.hot_data.module_ids);

        let handle = thread::spawn(move || {
            let frame_rate = 20;
            let frame_time = Duration::from_millis(1000 / frame_rate);
            //GameLoop
            loop {
                let before_frame = Instant::now();

                system.update(frame_time);

                let after_frame = Instant::now();

                let frame_duration = after_frame.duration_since(before_frame);

                if let Some(delta_time) = frame_time.checked_sub(frame_duration) {
                    std::thread::sleep(delta_time);
                } else {
                    println!(
                        "The frame took {:?}, too long couldn't sleep",
                        frame_duration
                    );
                }
            }
        });

        (handle, tx, arc)
    }

    fn update(&mut self, delta_time: Duration) {
        if let Ok(result) = self.channel.try_recv() {
            match result {
                SystemMessage::AddComponent(module_id, duration) => {
                    self.cold_data.add_timer(module_id, duration)
                }
                SystemMessage::UpdateComponent(module_id, duration) => {
                    self.cold_data.update_timer(module_id, duration)
                }
                SystemMessage::RemoveComponent(module_id) => self.cold_data.remove_timer(module_id),
                SystemMessage::StartTimer(module_id) => self.start_timer(module_id),
            }
        }

        self.hot_data.update_timers(delta_time);
    }

    fn start_timer(&mut self, module_id: Module) {
        for (index, id) in self.cold_data.module_ids.iter().enumerate() {
            if *id == module_id {
                self.hot_data
                    .start_timer(module_id, self.cold_data.durations[index]);
                break;
            }
        }
    }
}

struct ModuleCooldowns {
    module_ids: Vec<Module>,
    durations: Vec<Duration>,
}

impl ModuleCooldowns {
    fn new(capacity: usize) -> Self {
        Self {
            module_ids: Vec::with_capacity(capacity),
            durations: Vec::with_capacity(capacity),
        }
    }

    fn add_timer(&mut self, module_id: Module, duration: Duration) {
        self.module_ids.push(module_id);
        self.durations.push(duration);
    }

    fn update_timer(&mut self, module_id: Module, duration: Duration) {
        for (index, id) in self.module_ids.iter().enumerate() {
            if *id == module_id {
                self.durations[index] = duration;
                break;
            }
        }
    }

    fn remove_timer(&mut self, module_id: Module) {
        for (index, id) in self.module_ids.iter().enumerate() {
            if *id == module_id {
                self.module_ids.swap_remove(index);
                self.durations.swap_remove(index);
                break;
            }
        }
    }
}

struct ActiveCooldowns {
    module_ids: Arc<SharedIds>,
    timers: Vec<Duration>,
}

const ZERO_DURATION: Duration = Duration::from_nanos(0);

impl ActiveCooldowns {
    fn new(capacity: usize) -> Self {
        Self {
            module_ids: Arc::new(SharedIds::new(capacity)),
            timers: Vec::with_capacity(capacity),
        }
    }

    fn start_timer(&mut self, module_id: Module, time: Duration) {
        if let Ok(mut module_ids) = self.module_ids.data.write() {
            module_ids.push(module_id);
        }

        self.timers.push(time);
    }

    fn update_timers(&mut self, delta_time: Duration) {
        let mut i = self.timers.len();

        while i != 0 {
            i -= 1; //iterate in reverse because of swap_remove

            if let Some(result) = self.timers[i - 1].checked_sub(delta_time) {
                if result != ZERO_DURATION {
                    self.timers[i - 1] = result;

                    i -= 1;
                    continue;
                }
            }

            if let Ok(mut module_ids) = self.module_ids.data.write() {
                module_ids.swap_remove(i - 1);
            }

            self.timers.swap_remove(i - 1);
        }
    }
}

pub struct SharedIds {
    data: RwLock<Vec<Module>>,
}

impl SharedIds {
    fn new(capacity: usize) -> Self {
        Self {
            data: RwLock::new(Vec::with_capacity(capacity)),
        }
    }

    pub fn on_cooldown(&self, module_id: Module) -> bool {
        if let Ok(module_ids) = self.data.read() {
            for id in module_ids.iter() {
                if *id == module_id {
                    return true;
                }
            }

            return false;
        }

        true
    }
}

/* #[cfg(test)]
mod tests {
    use super::*;
    use crate::id_types::Module;
    use std::time::Duration;

    #[test]
    fn multi_timer() {}
} */

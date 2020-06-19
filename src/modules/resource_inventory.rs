use crate::id_types::ResourceHandle;
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;

pub enum SystemMessage {
    UpdateOrAdd(ResourceHandle, i32),
}

pub struct System {
    channel: Receiver<SystemMessage>,

    data: ResourceInventoryData,
}

impl System {
    pub fn init(capacity: usize) -> (JoinHandle<()>, Sender<SystemMessage>) {
        let (tx, rx) = mpsc::channel();

        let mut system = Self {
            channel: rx,

            data: ResourceInventoryData::with_capacity(capacity),
        };

        let handle = thread::spawn(move || {
            system.update_loop();
        });

        (handle, tx)
    }

    fn update_loop(&mut self) {
        while let Ok(message) = self.channel.recv() {
            match message {
                SystemMessage::UpdateOrAdd(resource_handle, quantity) => {
                    self.data.update_or_insert(resource_handle, quantity);
                }
            }
        }
    }
}

struct ResourceInventoryData {
    resources: HashMap<ResourceHandle, ResourceData>,
}

impl ResourceInventoryData {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            resources: HashMap::with_capacity(capacity),
        }
    }

    fn update_or_insert(&mut self, handle: ResourceHandle, delta: i32) {
        let resource_data = self.resources.get_mut(&handle);
        let quantity = match resource_data {
            Some(resource_data) => resource_data.quantity.get(),
            None => 0,
        };

        if delta.is_negative() {
            let result = quantity.saturating_sub(delta.abs() as u32);

            if let Some(quantity) = NonZeroU32::new(result) {
                self.resources.insert(handle, ResourceData { quantity });
                return;
            }
        } else if let Some(quantity) = NonZeroU32::new(quantity + delta as u32) {
            self.resources.insert(handle, ResourceData { quantity });
            return;
        }

        self.resources.remove(&handle);
    }
}

struct ResourceData {
    quantity: NonZeroU32,
    //can easily extend
}

#[cfg(test)]
mod tests {
    //use super::*;
    //use crate::id_types::Module;
    //use std::time::Duration;

    #[test]
    fn sat_neg_add() {}
}

use crate::id_types::{DatabaseId, ModuleHandle, Resource};
use crate::modules::crafting::ModuleData;
use std::collections::HashMap;
use std::num::NonZeroI32;
use std::num::NonZeroU32;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;

pub enum SystemMessage {
    AddModule(ModuleHandle, ModuleData),
    RemoveModule(ModuleHandle),

    UpdateDurability(ModuleHandle, i32),
}

pub struct System {
    channel: Receiver<SystemMessage>,

    data: ModuleInventoryData,
}

impl System {
    pub fn init(capacity: usize) -> (JoinHandle<()>, Sender<SystemMessage>) {
        let (tx, rx) = mpsc::channel();

        let mut system = Self {
            channel: rx,

            data: ModuleInventoryData::with_capacity(capacity),
        };

        let handle = thread::spawn(move || {
            system.update_loop();
        });

        (handle, tx)
    }

    fn update_loop(&mut self) {
        while let Ok(message) = self.channel.recv() {
            match message {
                SystemMessage::AddModule(module_handle, data) => {
                    self.data.modules.insert(module_handle, data);
                }
                SystemMessage::RemoveModule(module_handle) => {
                    self.data.modules.remove(&module_handle);
                }
                SystemMessage::UpdateDurability(module_handle, delta) => {
                    self.data.update_module_durability(&module_handle, delta);
                }
            }
        }
    }
}

struct ModuleInventoryData {
    //Random access only
    modules: HashMap<ModuleHandle, ModuleData>,
}

impl ModuleInventoryData {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            modules: HashMap::with_capacity(capacity),
        }
    }

    fn update_module_durability(&mut self, handle: &ModuleHandle, delta: i32) {
        let module_data = self.modules.get_mut(&handle);
        let module_data = match module_data {
            Some(module_data) => module_data,
            None => return,
        };

        if let Some(result) = NonZeroI32::new(delta) {
            module_data.resources.update_durability(result);
        }
    }
}

pub struct ModuleResources {
    //Iteration only
    resource_ids: Vec<Resource>,
    quantities: Vec<NonZeroU32>,
}

impl ModuleResources {
    fn new() -> Self {
        Self {
            resource_ids: Vec::with_capacity(5),
            quantities: Vec::with_capacity(5),
        }
    }

    fn update_durability(&mut self, total_change: NonZeroI32) {
        let sign = total_change.get().signum();

        let mut delta = total_change.get().abs() as u32 / self.quantities.len() as u32;
        let mut remainder = total_change.get().abs() as u32 % self.quantities.len() as u32; // 0 <= X <= indices.len() - 1

        for index in (self.quantities.len() - 1)..0 {
            //reverse iterate because swap_remove
            if remainder > 0 {
                delta += 1;
                remainder -= 1;
            }

            if let Some(delta) = NonZeroI32::new(sign * delta as i32) {
                self.update_resource_quantity(index, delta);
            }
        }
    }

    fn update_resource_quantity(&mut self, index: usize, quantity: NonZeroI32) {
        if quantity.get().is_positive() {
            if let Some(result) =
                NonZeroU32::new(self.quantities[index].get() + quantity.get() as u32)
            {
                self.quantities[index] = result;
                return;
            }
        }

        if let Some(result) = self.quantities[index]
            .get()
            .checked_sub(quantity.get().abs() as u32)
        {
            if let Some(result) = NonZeroU32::new(result) {
                self.quantities[index] = result;
                return;
            }
        }

        self.swap_remove(index);
    }

    fn swap_remove(&mut self, index: usize) {
        self.resource_ids.swap_remove(index);
        self.quantities.swap_remove(index);
    }
}

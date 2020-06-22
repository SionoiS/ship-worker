use crate::id_types::{DatabaseId, Module, Resource};
use std::collections::HashMap;
use std::num::NonZeroI32;
use std::num::NonZeroU32;

pub struct Modules {
    modules: HashMap<Module, ModuleData>,
}

impl Modules {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            modules: HashMap::with_capacity(capacity),
        }
    }

    pub fn update_module_durability(&mut self, module_id: &Module, delta: i32) {
        let module_data = self.modules.get_mut(module_id);
        let module_data = match module_data {
            Some(module_data) => module_data,
            None => return,
        };

        if module_data.resources.enough_durability(delta) {
            if let Some(result) = NonZeroI32::new(delta) {
                module_data.resources.update_durability(result);
            }
        } else {
            self.modules.remove(module_id);
        }
    }
}

struct ModuleData {
    name: String,
    creator: DatabaseId,
    properties: [u8; 5],
    resources: ModuleResources,
}

struct ModuleResources {
    resource_ids: Vec<Resource>,
    quantities: Vec<NonZeroU32>,
}

impl ModuleResources {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            resource_ids: Vec::with_capacity(capacity),
            quantities: Vec::with_capacity(capacity),
        }
    }

    fn enough_durability(&self, delta: i32) -> bool {
        if delta.is_negative() {
            let mut total = 0;

            for quantity in self.quantities.iter() {
                total += quantity.get();
            }

            total > delta.abs() as u32
        } else {
            true
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

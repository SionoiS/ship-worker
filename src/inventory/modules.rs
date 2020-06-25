use procedural_generation::id_types::{Module, Resource, User};
use std::collections::HashMap;
use std::num::NonZeroI32;
use std::num::NonZeroU32;

#[derive(Clone)]
pub struct Modules {
    pub modules: HashMap<Module, ModuleStats>,
}

#[derive(Clone)]
pub struct ModuleStats {
    name: String,
    creator: User,
    properties: Vec<u8>,
    resources: ModuleResources,
}

#[derive(Clone)]
pub struct ModuleResources {
    resource_ids: Vec<Resource>,
    quantities: Vec<NonZeroU32>,
}

impl Modules {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            modules: HashMap::with_capacity(capacity),
        }
    }

    pub fn add(&mut self, module_id: &Module, module: ModuleStats) {
        self.modules.insert(*module_id, module);
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

impl ModuleStats {
    pub fn new(name: String, creator: User, levels: &[u8], resources: ModuleResources) -> Self {
        let mut properties = Vec::with_capacity(levels.len());

        properties.copy_from_slice(levels);

        ModuleStats {
            name,
            creator,
            properties,
            resources,
        }
    }

    pub fn get_properties(&self) -> Vec<u8> {
        self.properties.to_vec()
    }
}

impl ModuleResources {
    pub fn new(input_resources: &[Resource], input_quantities: &[NonZeroU32]) -> Self {
        let mut resource_ids = Vec::with_capacity(input_resources.len());
        let mut quantities = Vec::with_capacity(input_quantities.len());

        resource_ids.copy_from_slice(input_resources);
        quantities.copy_from_slice(input_quantities);

        Self {
            resource_ids,
            quantities,
        }
    }

    fn enough_durability(&self, delta: i32) -> bool {
        if delta.is_positive() {
            return true;
        }

        let mut total = 0;

        for quantity in self.quantities.iter() {
            total += quantity.get();
        }

        total > delta.abs() as u32
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

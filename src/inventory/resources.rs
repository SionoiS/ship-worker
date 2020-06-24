use crate::id_types::Resource;
use std::collections::HashMap;
use std::num::NonZeroU32;

#[derive(Clone)]
pub struct Resources {
    resources: HashMap<Resource, NonZeroU32>,
}

impl Resources {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            resources: HashMap::with_capacity(capacity),
        }
    }

    pub fn has_enough(&self, resource_id: &Resource, quantity: NonZeroU32) -> bool {
        let res_quantity = self.resources.get(resource_id);
        match res_quantity {
            Some(res_quantity) => *res_quantity >= quantity,
            None => false,
        }
    }

    pub fn update_or_insert(&mut self, resource_id: &Resource, delta: i32) {
        let quantity = self.resources.get_mut(&resource_id);
        let quantity = match quantity {
            Some(quantity) => quantity.get(),
            None => 0,
        };

        if delta.is_negative() {
            let result = quantity.saturating_sub(delta.abs() as u32);

            if let Some(quantity) = NonZeroU32::new(result) {
                self.resources.insert(*resource_id, quantity);
            } else {
                self.resources.remove(&resource_id);
            }
        } else if let Some(quantity) = NonZeroU32::new(quantity + delta as u32) {
            self.resources.insert(*resource_id, quantity);
        }
    }
}

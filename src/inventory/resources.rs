use crate::id_types::Resource;
use std::collections::HashMap;
use std::num::NonZeroU32;

pub struct Resources {
    resources: HashMap<Resource, NonZeroU32>,
}

impl Resources {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            resources: HashMap::with_capacity(capacity),
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

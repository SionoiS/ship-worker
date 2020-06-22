use crate::id_types::{DatabaseId, Module, Ship};

fn craft_module(user_id: DatabaseId, ship_id: Ship, module_id: Module, properties: [u8; 5]) {
    //verify if user can craft this module with these properties

    //get resource requirements from lib

    //read cargo resources to verify requirements

    //send message to add new module and remove resource used
}

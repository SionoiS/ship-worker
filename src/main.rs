use crate::id_types::{DatabaseId, Module};
use std::time::Duration;

mod database;
mod id_types;
mod modules;
mod ships;
mod spatial_os;

fn main() {
    let mut handles = vec![];

    let (handle, spatial_os) = spatial_os::connexion::System::init();
    handles.push(handle);

    let (handle, database) = database::firestore::System::init();
    handles.push(handle);

    let (handle, system, ids) = ships::identifications::System::init(100);
    handles.push(handle);

    let (handle, system, exploration) = ships::exploration::System::init(100);
    handles.push(handle);

    let (handle, system) = ships::heartbeats::System::init(100, 30_000, spatial_os.clone());
    handles.push(handle);

    let (handle, system, positions) =
        ships::positions::System::init(100, 900_000, spatial_os, database);
    handles.push(handle);

    let (handle, system, cooldowns) = modules::cooldowns::System::init(1000);
    handles.push(handle);

    let (handle, system) = modules::sensors::System::init(100, positions, cooldowns, system);
    handles.push(handle);

    /*
    let sender_clone = mpsc::Sender::clone(&cooldown_system_sender);
    let arc_clone = Arc::clone(&cooldown_system_shared);

    let module_id = DatabaseId::from_string("hdkjdydjdiufjdneudohyg").unwrap();
    let id = Module::Sensor(module_id);

    let message =
        modules::cooldowns::SystemMessage::UpdateComponent(id, Duration::from_millis(250));

    if let Err(error) = cooldown_sender.send(message) {
        println!("{}", error);
    }

    if cooldown_shared.on_cooldown(id) {
        println!("On Cooldown!");
    } */
}

mod database;
mod id_types;
mod inventory;
mod modules;
mod ships;
mod spatial_os;

fn main() {
    let mut handles = vec![];

    let (handle, spatial_os) = spatial_os::connexion::System::init();
    handles.push(handle);

    let (handle, database) = database::firestore::System::init();
    handles.push(handle);

    let (handle, _id_system, identifiers) = ships::identifications::System::init(100);
    handles.push(handle);

    let (handle, _exploration_system, asteroids) = ships::exploration::System::init(100);
    handles.push(handle);

    let (handle, _heartbeat_system) =
        ships::heartbeats::System::init(100, 30_000, spatial_os.clone());
    handles.push(handle);

    let (handle, _positions_system, positions) =
        ships::positions::System::init(100, 900_000, spatial_os.clone(), database.clone());
    handles.push(handle);

    let (handle, cooldown_system, cooldowns) = modules::cooldowns::System::init(1000);
    handles.push(handle);

    //let (handle, _crafting_system, levels) = inventory::crafting::System::init(100);
    //handles.push(handle);

    let (handle, inventory_system) = inventory::System::init(
        100,
        spatial_os.clone(), /*, levels.clone()*/
        identifiers.clone(),
    );
    handles.push(handle);

    let (handle, _sensors_system) = modules::sensors::System::init(
        100,
        spatial_os.clone(),
        cooldown_system.clone(),
        inventory_system.clone(),
        positions.clone(),
        cooldowns.clone(),
    );
    handles.push(handle);

    let (handle, _scanners_system) = modules::scanners::System::init(
        100,
        spatial_os.clone(),
        cooldown_system.clone(),
        inventory_system.clone(),
        asteroids.clone(),
        cooldowns.clone(),
        identifiers.clone(),
    );
    handles.push(handle);

    let (handle, _samplers_system) = modules::samplers::System::init(
        100,
        spatial_os.clone(),
        cooldown_system.clone(),
        inventory_system.clone(),
        asteroids.clone(),
        cooldowns.clone(),
    );
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

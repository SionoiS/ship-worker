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

    let (handle, inventory_system, inventories) =
        inventory::System::init(100, spatial_os.clone(), identifiers.clone());
    handles.push(handle);

    let (handle, _sensors_system) = modules::sensors::System::init(
        100,
        spatial_os.clone(),
        cooldown_system.clone(),
        inventory_system.clone(),
        positions.clone(),
        cooldowns.clone(),
        inventories.clone(),
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
        inventories.clone(),
    );
    handles.push(handle);

    let (handle, _samplers_system) = modules::samplers::System::init(
        100,
        spatial_os.clone(),
        cooldown_system.clone(),
        inventory_system.clone(),
        asteroids.clone(),
        cooldowns.clone(),
        inventories.clone(),
    );
    handles.push(handle);

    let (handle, _damageable_system, _damageables) =
        ships::damageables::System::init(100, spatial_os.clone());
    handles.push(handle);
}

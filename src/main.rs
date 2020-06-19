use crate::id_types::{DatabaseId, Module};
use std::time::Duration;

mod heartbeats;
mod id_types;
mod modules;
mod positions;
mod spatial_os;

fn main() {
    let mut handles = vec![];

    let (handle, connexion) = spatial_os::connexion::System::init();
    handles.push(handle);

    let (handle, system) = heartbeats::System::init(100, 30_000, connexion.clone());
    handles.push(handle);

    let (handle, system, positions) = positions::System::init(100, 900_000, connexion);
    handles.push(handle);

    /* let (cooldown_handle, cooldown_sender, cooldown_shared) =
        modules::cooldowns::System::init(1000);
    handles.push(cooldown_handle);

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

use std::sync::mpsc::Sender;
use std::thread;

use crate::infrastructure::{
    ComponentUpdateAction, ComponentUpdateEvent, run_component_update_action,
};

pub fn run_component_update_worker(
    action: ComponentUpdateAction,
    proxy_url: Option<String>,
    tx: Sender<ComponentUpdateEvent>,
) {
    thread::spawn(move || {
        let event_tx = tx.clone();
        run_component_update_action(action, proxy_url, |event| {
            let _ = event_tx.send(event);
        });
    });
}

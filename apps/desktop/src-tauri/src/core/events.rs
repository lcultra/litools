use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum PluginEvent {
    CommandsAdded(String, Vec<String>),   // plugin_id, command_ids
    CommandsRemoved(String, Vec<String>), // plugin_id, command_ids
    CommandsUpdated(String, Vec<String>), // plugin_id, command_ids
    CommandsReplaced(String, usize),      // plugin_id, count
    PluginReloadStarted(String),          // plugin_id
    PluginReloadFinished(String),         // plugin_id
}

pub type EventHandler = Arc<dyn Fn(&PluginEvent) + Send + Sync>;

pub struct PluginEventBus {
    handlers: Mutex<Vec<EventHandler>>,
}

impl PluginEventBus {
    pub fn new() -> Self {
        Self {
            handlers: Mutex::new(Vec::new()),
        }
    }

    pub fn subscribe(&self, handler: EventHandler) {
        self.handlers.lock().unwrap().push(handler);
    }

    pub fn emit(&self, event: PluginEvent) {
        let handlers = self.handlers.lock().unwrap();
        for handler in handlers.iter() {
            handler(&event);
        }
    }
}

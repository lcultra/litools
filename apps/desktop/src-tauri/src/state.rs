use std::sync::Mutex;

use litools_core::{LitoolsApp, LitoolsResult};

pub struct AppState {
    app: Mutex<LitoolsApp>,
}

impl AppState {
    pub fn bootstrap() -> LitoolsResult<Self> {
        Ok(Self {
            app: Mutex::new(LitoolsApp::bootstrap_in_memory()?),
        })
    }

    pub fn app(&self) -> &Mutex<LitoolsApp> {
        &self.app
    }
}

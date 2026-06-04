use std::{path::Path, sync::Mutex};

use litools_core::{LitoolsApp, LitoolsResult};

pub struct AppState {
    app: Mutex<LitoolsApp>,
}

impl AppState {
    pub fn bootstrap(data_dir: impl AsRef<Path>) -> LitoolsResult<Self> {
        Ok(Self {
            app: Mutex::new(LitoolsApp::bootstrap(data_dir)?),
        })
    }

    pub fn app(&self) -> &Mutex<LitoolsApp> {
        &self.app
    }
}

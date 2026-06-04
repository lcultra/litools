use std::{
    path::Path,
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use litools_core::{LitoolsApp, LitoolsResult};

pub struct AppState {
    app: Mutex<LitoolsApp>,
    quitting: AtomicBool,
}

impl AppState {
    pub fn bootstrap(data_dir: impl AsRef<Path>) -> LitoolsResult<Self> {
        Ok(Self {
            app: Mutex::new(LitoolsApp::bootstrap(data_dir)?),
            quitting: AtomicBool::new(false),
        })
    }

    pub fn app(&self) -> &Mutex<LitoolsApp> {
        &self.app
    }

    pub fn request_quit(&self) {
        self.quitting.store(true, Ordering::SeqCst);
    }

    pub fn is_quitting(&self) -> bool {
        self.quitting.load(Ordering::SeqCst)
    }
}

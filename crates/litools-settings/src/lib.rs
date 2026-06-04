pub mod profile;
pub mod settings;
pub mod storage;

pub use settings::{
    AppSettings, DEFAULT_GLOBAL_HOTKEY, DEFAULT_RESULT_LIMIT, MAX_RESULT_LIMIT, PaletteSettings,
    SearchSettings, WindowSettings,
};

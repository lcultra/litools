#[derive(Clone, Debug)]
pub enum LaunchTarget {
    App(String),
    File(String),
}

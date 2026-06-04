#[derive(Clone, Debug)]
pub struct FileIndexRoot {
    pub path: String,
    pub exclusions: Vec<String>,
}

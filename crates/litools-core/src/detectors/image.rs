use async_trait::async_trait;
use litools_search::{Detection, InputDetector};

pub struct ImageDetector;

impl ImageDetector {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl InputDetector for ImageDetector {
    fn id(&self) -> &str {
        "image"
    }

    async fn detect(&self, _input: &str) -> Option<Detection> {
        // Phase 4A: 图片检测依赖附件管道，暂不通过纯文本检测。
        None
    }
}

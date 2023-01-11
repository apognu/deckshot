use crate::{GameScreenshot, Uploader};

#[derive(Clone)]
pub struct NoopUploader;

impl NoopUploader {
  pub fn build() -> Result<Self, anyhow::Error> {
    Ok(NoopUploader)
  }
}

#[async_trait]
impl Uploader for NoopUploader {
  fn name(&self) -> &'static str {
    "noop"
  }

  async fn upload<'a>(&'a self, _screenshot: &'a GameScreenshot) -> Result<&'a GameScreenshot, anyhow::Error> {
    unimplemented!();
  }
}

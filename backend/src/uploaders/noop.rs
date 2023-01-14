use anyhow::anyhow;
use rand::{thread_rng, Rng};

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

  async fn upload<'a>(&'a self, screenshot: &'a GameScreenshot) -> Result<&'a GameScreenshot, anyhow::Error> {
    match thread_rng().gen::<bool>() {
      true => Ok(screenshot),
      false => Err(anyhow!("upload failed!")),
    }
  }
}

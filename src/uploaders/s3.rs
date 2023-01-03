use anyhow::Context;
use s3::{creds::Credentials, Bucket, Region};
use serde::Deserialize;
use tokio::{fs::File, io::BufReader};

use crate::{GameScreenshot, Uploader};

pub struct S3Uploader {
  bucket: Bucket,
}

#[derive(Clone, Deserialize)]
pub struct S3Config {
  pub endpoint: String,
  pub region: Option<String>,
  pub access_key_id: String,
  pub secret_access_key: String,
  pub bucket: String,
}

impl S3Uploader {
  pub fn build(config: S3Config) -> Result<Self, anyhow::Error> {
    let bucket = Bucket::new(
      &config.bucket,
      Region::Custom {
        region: config.region.unwrap_or_default(),
        endpoint: config.endpoint,
      },
      Credentials::new(Some(&config.access_key_id), Some(&config.secret_access_key), None, None, None)?,
    )?
    .with_path_style();

    Ok(S3Uploader { bucket })
  }
}

#[async_trait]
impl Uploader for S3Uploader {
  async fn upload(&self, screenshot: GameScreenshot) -> Result<GameScreenshot, anyhow::Error> {
    let dest = screenshot.dest_name().await?;
    let file = File::open(&screenshot.path).await?;
    let mut reader = BufReader::new(file);

    self.bucket.put_object_stream(&mut reader, dest.to_string_lossy()).await.context("could not upload screenshot")?;

    Ok(screenshot)
  }
}

use std::{path::PathBuf, sync::Arc};

use anyhow::Context;
use serde::Deserialize;

use crate::uploaders::{
  dropbox::{DropboxConfig, DropboxUploader},
  gdrive::{GoogleDriveConfig, GoogleDriveUploader},
  imgur::{ImgurConfig, ImgurUploader},
  noop::NoopUploader,
  onedrive::{OneDriveConfig, OneDriveUploader},
  s3::{S3Config, S3Uploader},
  Uploader,
};

#[derive(Clone, Deserialize)]
#[serde(tag = "kind")]
pub enum UploaderKind {
  Noop,
  S3(S3Config),
  GoogleDrive(GoogleDriveConfig),
  Dropbox(DropboxConfig),
  OneDrive(OneDriveConfig),
  Imgur(ImgurConfig),
}

#[derive(Clone, Deserialize)]
pub struct Config {
  #[serde(default = "default_deckshot_path")]
  pub deckshot_path: PathBuf,
  #[serde(default = "default_screenshot_path")]
  pub screenshots_path: PathBuf,
  pub uploader: UploaderKind,
  #[serde(default = "default_retrier_interval")]
  pub retrier_interval: u64,
}

fn default_deckshot_path() -> PathBuf {
  "/home/deck/.local/share/deckshot".into()
}

fn default_screenshot_path() -> PathBuf {
  "/home/deck/.local/share/Steam/userdata".into()
}

const fn default_retrier_interval() -> u64 {
  60
}

impl Config {
  pub async fn uploader(&self) -> Result<Arc<Box<dyn Uploader>>, anyhow::Error> {
    let uploader: Box<dyn Uploader> = match self.uploader {
      UploaderKind::Noop => Box::new(NoopUploader::build()?),
      UploaderKind::S3(ref config) => Box::new(S3Uploader::build(config.clone())?),
      UploaderKind::GoogleDrive(ref config) => Box::new(GoogleDriveUploader::build(config.clone()).await?),
      UploaderKind::Dropbox(ref config) => Box::new(DropboxUploader::build(self, config.clone()).await?),
      UploaderKind::OneDrive(ref config) => Box::new(OneDriveUploader::build(self, config.clone()).await?),
      UploaderKind::Imgur(ref config) => Box::new(ImgurUploader::build(self, config.clone())?),
    };

    Ok(Arc::new(uploader))
  }
}

pub fn read_config(path: Option<&PathBuf>) -> Result<Config, anyhow::Error> {
  let default: PathBuf = default_deckshot_path().join("deckshot.yml");
  let path = path.unwrap_or(&default);
  let file = std::fs::File::open(path).context(format!("could not open configuration file: {}", path.display()))?;

  serde_yaml::from_reader::<_, Config>(file).context(format!("could not parse configuration file: {}", path.display()))
}

use std::fs::File;

use anyhow::Context;
use google_drive3::{
  api::{DriveHub, File as RemoteFile},
  client::oauth2 as helpers,
  hyper::{self, client::HttpConnector},
  hyper_rustls::{self, HttpsConnector},
  oauth2,
};
use serde::Deserialize;

use crate::{GameScreenshot, Uploader};

pub struct GoogleDriveUploader {
  hub: DriveHub<HttpsConnector<HttpConnector>>,
  folder: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GoogleDriveConfig {
  pub private_key_file: String,
  pub folder: String,
}

impl GoogleDriveUploader {
  pub async fn build(config: GoogleDriveConfig) -> Result<Self, anyhow::Error> {
    let pkey = helpers::read_service_account_key(config.private_key_file).await?;
    let auth = oauth2::ServiceAccountAuthenticator::builder(pkey).build().await.context("could not parse private key")?;

    let hub = DriveHub::new(
      hyper::Client::builder().build(hyper_rustls::HttpsConnectorBuilder::new().with_native_roots().https_or_http().enable_http1().enable_http2().build()),
      auth,
    );

    Ok(GoogleDriveUploader { hub, folder: config.folder })
  }
}

#[async_trait]
impl Uploader for GoogleDriveUploader {
  async fn upload(&self, screenshot: GameScreenshot) -> Result<GameScreenshot, anyhow::Error> {
    let filename = screenshot.file_name()?;
    let game = screenshot.game_name().await;

    let (_, folders) = self
      .hub
      .files()
      .list()
      .q(&format!("mimeType = 'application/vnd.google-apps.folder' and '{}' in parents and name = '{game}'", self.folder))
      .doit()
      .await
      .context("could not find game folder")?;

    let folder = if let Some(folders) = folders.files && !folders.is_empty() {
      folders[0].clone()
    } else {
      let remote = RemoteFile { name: Some(game), parents: Some(vec![self.folder.clone()]), mime_type: Some("application/vnd.google-apps.folder".to_string()), ..Default::default() };

      let (_, file) = self.hub.files().create(remote).upload(std::io::empty(), "application/vnd.google-apps.folder".parse().unwrap()).await.context("could not create game folder")?;

      file
    };

    let file = File::open(&screenshot.path)?;

    let remote = RemoteFile {
      parents: Some(vec![folder.id.unwrap()]),
      name: Some(filename.to_string_lossy().into_owned()),
      ..Default::default()
    };

    self.hub.files().create(remote).upload(file, "image/jpeg".parse().unwrap()).await.context("could not upload file")?;

    Ok(screenshot)
  }
}

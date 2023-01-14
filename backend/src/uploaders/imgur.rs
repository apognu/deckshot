use std::{
  fs::File,
  io,
  time::{Duration, SystemTime},
};

use anyhow::anyhow;
use imgurs_client::{
  client::BasicClient,
  endpoints::authorization::{AuthenticationClient, AuthenticationRegisteredClient, Method},
  traits::{Client, RegisteredClient},
};
use imgurs_model::model::authorization::{AccessToken, AuthorizationCode, ClientID, ClientSecret, RefreshToken};
use serde::Deserialize;

use crate::{
  config::Config,
  database::{load_token, save_token},
  uploaders::prompt_authorization_code,
  GameScreenshot, Uploader,
};

pub struct ImgurUploader {
  config: Config,
  client: BasicClient,
}

#[derive(Clone, Deserialize)]
pub struct ImgurConfig {
  pub client_id: String,
  pub client_secret: String,
  pub redirect_uri: String,
}

impl ImgurUploader {
  pub fn build(deckshot: &Config, config: ImgurConfig) -> Result<ImgurUploader, anyhow::Error> {
    let client = BasicClient::new(ClientID(config.client_id), ClientSecret(config.client_secret)).map_err(|err| anyhow!(err.to_string()))?;

    Ok(ImgurUploader { config: deckshot.clone(), client })
  }
}

#[async_trait]
impl Uploader for ImgurUploader {
  fn name(&self) -> &'static str {
    "imgur"
  }

  async fn upload<'a>(&'a self, screenshot: &'a GameScreenshot) -> Result<&'a GameScreenshot, anyhow::Error> {
    let access_token = AccessToken(load_token(&self.config, "imgur-access-token").await?);
    let refresh_token = RefreshToken(load_token(&self.config, "imgur-refresh-token").await?);
    let expires_in = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(load_token(&self.config, "imgur-expires-in").await?.parse::<u64>()?);

    let auth = self
      .client
      .clone()
      .with_tokens(access_token.clone(), refresh_token, expires_in)
      .map_err(|err| anyhow!(err.to_string()))?;

    let auth = auth.with_fresh_tokens().await?;

    if auth.get_authentication_settings().access_token != access_token {
      let expires_in = (SystemTime::now() + Duration::from_secs(14 * 24 * 60 * 60)).duration_since(SystemTime::UNIX_EPOCH)?.as_secs();

      save_token(&self.config, "imgur-access-token", &auth.get_authentication_settings().access_token.0).await?;
      save_token(&self.config, "imgur-refresh-token", &auth.get_authentication_settings().refresh_token.0).await?;
      save_token(&self.config, "imgur-expires-in", &expires_in.to_string()).await?;
    }

    let client = auth.get_client();

    let mut file = File::open(&screenshot.path)?;
    let mut buffer = Vec::new();

    let b64 = base64::engine::GeneralPurpose::new(&base64::alphabet::STANDARD, base64::engine::general_purpose::PAD);

    {
      let mut encoder = base64::write::EncoderWriter::new(&mut buffer, &b64);
      io::copy(&mut file, &mut encoder)?;
    }

    let body = [
      ("type", "base64"),
      ("image", &String::from_utf8_lossy(&buffer)),
      ("name", &format!("{} - {}", screenshot.game_name().await, screenshot.file_name()?.to_string_lossy())),
    ];

    match client.post("https://api.imgur.com/3/upload").headers(auth.get_headers()?).form(&body).send().await?.status().as_u16() {
      200 => Ok(screenshot),
      code => Err(anyhow!(format!("upload responded with code {code}"))),
    }
  }

  async fn auth(&self) -> Result<(), anyhow::Error> {
    let url = self.client.get_authentication_url(Method::AuthorizationCode, None)?;
    let code = prompt_authorization_code(url.as_str())?;
    let tokens = self.client.authorization_by_authorization_code(AuthorizationCode(code)).await?.content.result()?;
    let expires_in = (SystemTime::now() + Duration::from_secs(14 * 24 * 60 * 60)).duration_since(SystemTime::UNIX_EPOCH)?.as_secs();

    save_token(&self.config, "imgur-access-token", &tokens.access_token.0).await?;
    save_token(&self.config, "imgur-refresh-token", &tokens.refresh_token.0).await?;
    save_token(&self.config, "imgur-expires-in", &expires_in.to_string()).await?;

    Ok(())
  }
}

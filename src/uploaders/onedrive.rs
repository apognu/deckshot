use oauth2::{
  basic::BasicClient, reqwest::async_http_client, AccessToken, AuthType, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, RefreshToken, Scope, TokenResponse, TokenUrl,
};
use onedrive_api::{option::DriveItemPutOption, ConflictBehavior, DriveLocation, FileName, ItemLocation};
use serde::Deserialize;
use tokio::{fs::File, io::AsyncReadExt};

use crate::{
  config::Config,
  database::{load_token, save_token},
  uploaders::prompt_authorization_code,
  GameScreenshot, Uploader,
};

pub struct OneDriveUploader {
  config: Config,
  client: BasicClient,
  folder: Option<String>,
}

#[derive(Clone, Deserialize)]
pub struct OneDriveConfig {
  pub client_id: String,
  pub client_secret: String,
  pub redirect_uri: String,
  pub folder: Option<String>,
}

impl OneDriveUploader {
  pub async fn build(deckshot: &Config, config: OneDriveConfig) -> Result<Self, anyhow::Error> {
    let client = BasicClient::new(
      ClientId::new(config.client_id.clone()),
      Some(ClientSecret::new(config.client_secret.clone())),
      AuthUrl::new("https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize".to_string())?,
      Some(TokenUrl::new("https://login.microsoftonline.com/consumers/oauth2/v2.0/token".to_string())?),
    )
    .set_auth_type(AuthType::RequestBody)
    .set_redirect_uri(RedirectUrl::new(config.redirect_uri.clone())?);

    Ok(OneDriveUploader {
      config: deckshot.clone(),
      client,
      folder: config.folder,
    })
  }

  async fn try_upload(&self, screenshot: GameScreenshot) -> Result<GameScreenshot, anyhow::Error> {
    let token = AccessToken::new(load_token(&self.config, "onedrive-access-token").await?);

    let game = screenshot.game_name().await;
    let filename = FileName::new(screenshot.file_name()?.to_str().unwrap()).unwrap();

    let mut file = File::open(&screenshot.path).await?;
    let mut buffer = Vec::new();

    file.read_to_end(&mut buffer).await?;

    let drive = onedrive_api::OneDrive::new(token.secret(), DriveLocation::me());

    if let Some(ref folder) = self.folder {
      drive
        .create_folder_with_option(
          ItemLocation::root(),
          FileName::new(folder).unwrap(),
          DriveItemPutOption::new().conflict_behavior(ConflictBehavior::Replace),
        )
        .await?;
    }

    let root_folder_path = self.folder.clone().map(|path| format!("/{path}")).unwrap_or_else(|| "/".to_string());
    let root_folder = ItemLocation::from_path(&root_folder_path).unwrap();

    let folder = drive
      .create_folder_with_option(root_folder, FileName::new(&game).unwrap(), DriveItemPutOption::new().conflict_behavior(ConflictBehavior::Replace))
      .await?;

    let folder_id = folder.id.unwrap();
    let item = ItemLocation::child_of_id(&folder_id, filename);

    drive.upload_small(item, buffer).await?;

    Ok(screenshot)
  }
}

#[async_trait]
impl Uploader for OneDriveUploader {
  fn name(&self) -> &'static str {
    "Microsoft OneDrive"
  }

  async fn upload(&self, screenshot: GameScreenshot) -> Result<GameScreenshot, anyhow::Error> {
    match self.try_upload(screenshot).await {
      Ok(screenshot) => Ok(screenshot),

      Err(err) => {
        if err.downcast_ref::<onedrive_api::Error>().is_some() {
          let refresh = RefreshToken::new(load_token(&self.config, "onedrive-refresh-token").await?);
          let tokens = self.client.exchange_refresh_token(&refresh).request_async(async_http_client).await?;

          save_token(&self.config, "onedrive-access-token", tokens.access_token().secret()).await?;
        }

        Err(err)
      }
    }
  }

  async fn auth(&self) -> Result<(), anyhow::Error> {
    let (url, _) = self
      .client
      .authorize_url(CsrfToken::new_random)
      .add_scope(Scope::new("offline_access".to_string()))
      .add_scope(Scope::new("Files.ReadWrite".to_string()))
      .url();

    let code = prompt_authorization_code(url.as_str())?;
    let tokens = self.client.exchange_code(AuthorizationCode::new(code)).request_async(async_http_client).await?;

    save_token(&self.config, "onedrive-access-token", tokens.access_token().secret()).await?;
    save_token(&self.config, "onedrive-refresh-token", tokens.refresh_token().unwrap().secret()).await?;

    Ok(())
  }
}

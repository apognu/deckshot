use dropbox_sdk::{
  default_client::{NoauthDefaultClient, UserAuthDefaultClient},
  files::{self, UploadArg},
  oauth2::{Authorization, AuthorizeUrlBuilder, Oauth2Type, PkceCode},
};
use serde::Deserialize;
use tokio::{fs::File, io::AsyncReadExt};

use crate::{
  config::Config,
  database::{load_token, save_token},
  uploaders::prompt_authorization_code,
  GameScreenshot, Uploader,
};

pub struct DropboxUploader {
  config: Config,
  client_id: String,
  folder: Option<String>,
}

#[derive(Clone, Deserialize)]
pub struct DropboxConfig {
  pub client_id: String,
  pub folder: Option<String>,
}

impl DropboxUploader {
  pub async fn build(deckshot: &Config, config: DropboxConfig) -> Result<Self, anyhow::Error> {
    Ok(DropboxUploader {
      config: deckshot.clone(),
      client_id: config.client_id.clone(),
      folder: config.folder,
    })
  }
}

#[async_trait]
impl Uploader for DropboxUploader {
  async fn upload(&self, screenshot: GameScreenshot) -> Result<GameScreenshot, anyhow::Error> {
    let token = load_token(&self.config, "dropbox").await?;
    let auth = Authorization::load(self.client_id.clone(), &token).unwrap();
    let client = UserAuthDefaultClient::new(auth);

    let dest = if let Some(ref folder) = self.folder {
      format!("/{}/{}", folder, screenshot.dest_name().await?.to_string_lossy())
    } else {
      format!("/{}", screenshot.dest_name().await?.to_string_lossy())
    };

    let mut file = File::open(&screenshot.path).await?;
    let mut buffer = Vec::new();

    file.read_to_end(&mut buffer).await?;

    let args = UploadArg::new(dest);

    files::upload(&client, &args, &buffer)??;

    Ok(screenshot)
  }

  async fn auth(&self) -> Result<(), anyhow::Error> {
    let pkce = PkceCode::new();
    let flow = Oauth2Type::PKCE(pkce);
    let url = AuthorizeUrlBuilder::new(&self.client_id, &flow).build();
    let code = prompt_authorization_code(url.as_str())?;

    let mut auth = Authorization::from_auth_code(self.client_id.clone(), flow, code, None);
    auth.obtain_access_token(NoauthDefaultClient::default())?;

    save_token(&self.config, "dropbox", &auth.save().unwrap()).await?;

    Ok(())
  }
}

use serde::Deserialize;
use serenity::{self, async_trait, model::prelude::ChannelId, prelude::*};

use crate::{config::Config, GameScreenshot, Uploader};

pub struct DiscordUploader {
  client: Client,
  channel: u64,
  username: Option<String>,
}

#[derive(Clone, Deserialize)]
pub struct DiscordConfig {
  token: String,
  channel: u64,
  username: Option<String>,
}

impl DiscordUploader {
  pub async fn build(_deckshot: &Config, config: DiscordConfig) -> Result<Self, anyhow::Error> {
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let client = Client::builder(config.token.clone(), intents).await.expect("Error creating client");

    Ok(DiscordUploader {
      client,
      channel: config.channel,
      username: config.username,
    })
  }
}

#[async_trait]
impl Uploader for DiscordUploader {
  fn name(&self) -> &'static str {
    "Discord"
  }

  async fn upload<'a>(&'a self, screenshot: &'a GameScreenshot) -> Result<&'a GameScreenshot, anyhow::Error> {
    let game_name = screenshot.game_name().await;

    let http = self.client.cache_and_http.http.clone();
    let channel = ChannelId(self.channel);

    let text = match self.username {
      Some(ref username) => format!("{username} took a new screenshot from {game_name}"),
      None => format!("New screenshot from {game_name}"),
    };

    channel.send_message(&http, |message| message.content(text).add_file(screenshot.path.as_path())).await?;

    Ok(screenshot)
  }
}

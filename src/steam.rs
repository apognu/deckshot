use std::{
  collections::HashMap,
  ffi::OsStr,
  path::{Path, PathBuf},
};

use anyhow::anyhow;
use serde::Deserialize;

#[derive(Deserialize)]
struct GameInformationResponse {
  data: GameInformation,
}

#[derive(Deserialize)]
struct GameInformation {
  name: String,
}

pub struct GameScreenshot {
  pub game_id: u64,
  pub path: PathBuf,
}

impl GameScreenshot {
  pub fn file_name(&self) -> Result<&OsStr, anyhow::Error> {
    self.path.file_name().ok_or_else(|| anyhow!("could not determine file name"))
  }

  pub async fn game_name(&self) -> String {
    let url = format!("https://store.steampowered.com/api/appdetails?appids={}", self.game_id);

    if let Ok(response) = reqwest::get(url).await {
      if let Ok(payload) = response.json::<HashMap<u64, GameInformationResponse>>().await {
        if let Some(game) = payload.get(&self.game_id) {
          return game.data.name.clone();
        }
      }
    }

    "UNKNOWN_GAME".to_string()
  }

  pub async fn dest_name(&self) -> Result<PathBuf, anyhow::Error> {
    Ok(Path::new(&self.game_name().await).join(self.file_name()?))
  }
}

impl From<&Path> for GameScreenshot {
  fn from(path: &Path) -> Self {
    let game_id = path.iter().nth(10).unwrap_or_default().to_string_lossy().parse::<u64>().unwrap_or_default();

    GameScreenshot { game_id, path: path.to_owned() }
  }
}

use std::{
  collections::HashMap,
  ffi::OsStr,
  path::{Path, PathBuf},
};

use anyhow::anyhow;
use serde::Deserialize;

use crate::{database::Db, Uploader};

const APPS: &[(u64, &str)] = &[(7, "Steam UI")];

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
    if let Ok(name) = APPS.binary_search_by(|(id, _)| id.cmp(&self.game_id)).map(|id| APPS[id].1) {
      return name.to_string();
    }

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

  pub async fn upload(&self, uploader: &dyn Uploader, db: Db) -> Result<&GameScreenshot, anyhow::Error> {
    match uploader.upload(self).await {
      Ok(_) => Ok(self),

      Err(err) => match self.save(db).await {
        Ok(()) => Err(err),
        Err(save_err) => Err(save_err.context(err)),
      },
    }
  }

  pub async fn save(&self, db: Db) -> Result<(), anyhow::Error> {
    let mut db = db.lock().await;

    db.ladd("screenshots", &self.path.to_string_lossy()).ok_or_else(|| anyhow!("could not save screenshot"))?;

    Ok(())
  }
}

impl From<&Path> for GameScreenshot {
  fn from(path: &Path) -> Self {
    let game_id = path.iter().rev().nth(2).and_then(|id| id.to_string_lossy().parse::<u64>().ok()).unwrap_or(0);

    GameScreenshot { game_id, path: path.to_owned() }
  }
}

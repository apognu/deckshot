#![feature(async_closure, let_chains)]

#[macro_use]
extern crate async_trait;

mod config;
mod database;
mod steam;
mod uploaders;

use std::{
  env,
  path::{Path, PathBuf},
  sync::mpsc::channel,
  time::Duration,
};

use anyhow::Context;
use clap::{arg, value_parser, Command};
use kvlogger::*;
use notify::{
  event::{AccessKind, AccessMode},
  Event, EventKind, RecursiveMode, Watcher,
};

use crate::{steam::GameScreenshot, uploaders::Uploader};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
  if env::var("RUST_LOG").is_err() {
    env::set_var("RUST_LOG", "deckshot=info");
  }

  KvLoggerBuilder::default().init()?;

  let args = Command::new("deckshot")
    .arg(arg!(-c --config <FILE> "Location of configuration file").required(false).value_parser(value_parser!(PathBuf)))
    .subcommand(Command::new("auth").about("Launch an interactive authentication process"))
    .get_matches();

  let config = config::read_config(args.get_one::<PathBuf>("config"))?;
  let uploader = config.uploader().await.context("could not build uploader configuration")?;

  kvlog!(Info, format!("initialized uploader: {}", uploader.name()));

  if args.subcommand_matches("auth").is_some() {
    uploader.auth().await?;

    return Ok(());
  }

  let (tx, rx) = channel::<Event>();

  let mut watcher = notify::recommended_watcher(move |event| {
    if let Ok(event) = event {
      let _ = tx.send(event);
    }
  })?;

  watcher
    .watch(Path::new(&config.screenshots_path), RecursiveMode::Recursive)
    .context(format!("could not watch screenshot directory: {}", config.screenshots_path.display()))?;

  let db = database::init_db(&config).context("could not initialize database")?;

  tokio::spawn({
    let db = db.clone();
    let uploader = uploader.clone();

    async move {
      loop {
        let mut paths: Vec<String> = Vec::new();

        {
          let db = db.lock().await;
          let data = db.liter("screenshots");

          for item in data {
            if let Some(path) = item.get_item::<String>() {
              kvlog!(Info, "uploading failed screenshot", {
                  "path" => path
              });

              paths.push(path);
            }
          }
        }

        for (index, path) in paths.iter().enumerate() {
          let screenshot: GameScreenshot = PathBuf::from(path).as_path().into();

          match screenshot.upload(&**uploader, db.clone()).await {
            Ok(screenshot) => {
              kvlog!(Info, "screenshot uploaded", {
                  "path" => screenshot.path.display(),
                  "game" => screenshot.game_name().await
              });
            }

            Err(err) => {
              kvlog!(Error, "could not upload screenshot", {
                  "error" => format!("{err:#}")
              });
            }
          }

          db.lock().await.lpop::<String>("screenshots", index);
        }

        tokio::time::sleep(Duration::from_secs(config.retrier_interval)).await;
      }
    }
  });

  while let Ok(event) = rx.recv() {
    if event.kind == EventKind::Access(AccessKind::Close(AccessMode::Write)) {
      for path in event.paths {
        let screenshot: GameScreenshot = path.as_path().into();
        let lossy_path = screenshot.path.to_string_lossy();

        if lossy_path.ends_with(".jpg") && !lossy_path.contains("thumbnail") {
          match screenshot.upload(&**uploader, db.clone()).await {
            Ok(screenshot) => {
              kvlog!(Info, "screenshot uploaded", {
                  "path" => screenshot.path.display(),
                  "game" => screenshot.game_name().await
              });
            }

            Err(err) => {
              kvlog!(Error, "could not upload screenshot", {
                  "error" => format!("{err:#}")
              });
            }
          }
        }
      }
    }
  }

  Ok(())
}

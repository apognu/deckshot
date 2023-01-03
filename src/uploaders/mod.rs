pub mod dropbox;
pub mod gdrive;
pub mod noop;
pub mod onedrive;
pub mod s3;

use std::io::{self, Write};

use anyhow::Context;

use crate::GameScreenshot;

#[async_trait]
pub trait Uploader: Sync + Send {
  async fn upload(&self, screenshot: GameScreenshot) -> Result<GameScreenshot, anyhow::Error>;

  async fn auth(&self) -> Result<(), anyhow::Error> {
    unimplemented!();
  }
}

pub fn prompt_authorization_code(authorize_url: &str) -> Result<String, anyhow::Error> {
  println!("Open the following URL into your Web browser to authenticate, then input the generated code:");
  println!("{authorize_url}");

  let mut code = String::new();

  print!("Code: ");
  io::stdout().flush()?;
  io::stdin().read_line(&mut code).context("could not read code")?;

  Ok(code.trim().to_string())
}

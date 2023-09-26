extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use anyhow::Result;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt, StreamExt,
};
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use serde_derive::{Deserialize, Serialize};
use teloxide::{requests::Requester, types::InputFile, Bot};

use tokio::fs;
use tokio::time::{sleep, Duration};

#[derive(Default, Debug, Serialize, Deserialize)]
struct Config {
    path: String,
    token: String,
    chat_id: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "INFO");
    pretty_env_logger::init();

    let cfg: Config = confy::load_path(Path::new("./config.toml"))?;
    if cfg.path == "" {
        error!("please specify a path");

        return Ok(());
    }

    info!("watching {}", cfg.path);

    futures::executor::block_on(async {
        if let Err(e) = async_watch(cfg).await {
            error!("error: {:?}", e)
        }
    });

    Ok(())
}

fn async_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (mut tx, rx) = channel(1);

    let watcher = RecommendedWatcher::new(
        move |res| {
            futures::executor::block_on(async {
                tx.send(res).await.unwrap();
            })
        },
        notify::Config::default(),
    )?;

    Ok((watcher, rx))
}

async fn upload_file(path: PathBuf, chat_id: String, token: String) -> Result<()> {
    info!("screenshot taken: {:?}", path);

    sleep(Duration::from_millis(1500)).await;

    let temp_dir = tempdir()?;
    let temp_screenshot = Path::join(temp_dir.path(), "screenshot.png");
    fs::copy(path, &temp_screenshot).await?;

    let data = fs::read(temp_screenshot).await?;
    let bot = Bot::new(token);

    bot.send_photo(chat_id, InputFile::memory(data)).await?;

    fs::remove_dir_all(temp_dir).await?;
    drop(bot);

    info!("screenshot uploaded!");

    Ok(())
}

async fn async_watch(cfg: Config) -> Result<()> {
    let (mut watcher, mut rx) = async_watcher()?;
    let path = Path::new(&cfg.path);

    watcher.watch(path, RecursiveMode::Recursive)?;

    while let Some(res) = rx.next().await {
        match res {
            Ok(event) => {
                if !event.kind.is_create() {
                    continue;
                }

                for path in event.paths {
                    match upload_file(path, cfg.chat_id.to_string(), cfg.token.to_string()).await {
                        Ok(()) => (),
                        Err(e) => error!("upload error: {:?}", e),
                    };
                }
            }
            Err(e) => error!("watch error: {:?}", e),
        }
    }

    Ok(())
}

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use anyhow::{anyhow, Result};
use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt, StreamExt,
};
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use serde_derive::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    process,
};
use sysinfo::{ProcessExt, System, SystemExt};
use teloxide::{requests::Requester, types::InputFile, Bot};

use tokio::fs;
use tokio::time::{sleep, Duration};

#[derive(Default, Debug, Serialize, Deserialize)]
struct Config {
    path: Vec<String>,
    token: String,
    chat_id: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "INFO");
    pretty_env_logger::init_timed();

    let cfg: Config = confy::load_path(Path::new("./config.toml"))?;

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

async fn read_unlocked(path: &PathBuf) -> Result<Vec<u8>> {
    let mut data: Option<Vec<u8>> = None;
    for _ in 0..100 {
        match fs::read(&path).await {
            Ok(d) => {
                data = Some(d);
                break;
            }
            Err(e) => {
                sleep(Duration::from_millis(50)).await;

                trace!("error when trying to open file: {:?}; err: {}", path, e);

                continue;
            }
        }
    }

    if let Some(value) = data {
        Ok(value)
    } else {
        Err(anyhow!("failed to open file: {:?}", path))
    }
}

async fn upload_file(path: PathBuf, chat_id: String, token: String) -> Result<()> {
    let file_name = path.file_name().unwrap().to_owned();
    info!("new file detected: {:?}", file_name);

    let bot = Bot::new(token);
    let data = read_unlocked(&path).await?;

    match path.extension().unwrap_or_default().to_str().unwrap() {
        "png" | "jpg" | "jpeg" => bot.send_photo(chat_id, InputFile::memory(data)).await?,
        "mp4" | "gif" | "mkv" => bot.send_video(chat_id, InputFile::memory(data)).await?,
        _ => bot.send_document(chat_id, InputFile::memory(data)).await?,
    };

    drop(bot);

    info!("file uploaded!: {:?}", file_name);

    Ok(())
}

async fn monitor_processes() {
    let mut monitor_found: Option<bool> = None;

    'l: loop {
        let s = System::new_all();
        for _ in s.processes_by_name("vrmonitor") {
            monitor_found = Some(true);
            continue 'l;
        }

        if monitor_found.is_none() {
            warn!("no vrmonitor.exe process was found during startup, program will not close when you exit VR");

            break;
        }

        monitor_found = Some(false);

        if let Some(v) = monitor_found {
            if v == false {
                warn!("vrmonitor.exe was closed");

                process::exit(0);
            }
        }
    }
}

async fn async_watch(cfg: Config) -> Result<()> {
    let (mut watcher, mut rx) = async_watcher()?;

    tokio::spawn(async { monitor_processes().await });

    for path in cfg.path {
        info!("watching {}", &path);

        let path = Path::new(&path);
        watcher.watch(path, RecursiveMode::Recursive)?;
    }

    while let Some(res) = rx.next().await {
        match res {
            Ok(event) => {
                if !event.kind.is_create() {
                    continue;
                }

                for path in event.paths {
                    tokio::spawn(upload_file(
                        path,
                        cfg.chat_id.to_string(),
                        cfg.token.to_string(),
                    ));
                }
            }
            Err(e) => error!("watch error: {:?}", e),
        }
    }

    Ok(())
}

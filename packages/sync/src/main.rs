use crate::adapters::services::{
    card_source_init, card_storage_init, emoji_store_init, image_store_init,
};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::domain::{bulk, migrate, spoilers};
#[cfg(feature = "local-dev")]
use dotenv::dotenv;

pub mod adapters;
pub mod domain;
pub mod ports;

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Spoilers,
    Bulk,
    /// One-off: convert a legacy PNG image library into the WebP layout.
    /// Needs no database, Scryfall or Discord access.
    MigrateImages {
        /// Legacy library root (read-only), containing images/ and illustrations/
        #[arg(long)]
        from: PathBuf,
        /// Target library root (the card-images volume)
        #[arg(long)]
        to: PathBuf,
    },
}

#[tokio::main]
async fn main() {
    #[cfg(feature = "local-dev")]
    dotenv().ok();

    env_logger::init();

    let cli = Cli::parse();

    if let Command::MigrateImages { from, to } = cli.command {
        match tokio::task::spawn_blocking(move || migrate::migrate_images(&from, &to)).await {
            Ok(Ok(_)) => return,
            Ok(Err(why)) => log::error!("Image migration failed: {why}"),
            Err(why) => log::error!("Image migration panicked: {why}"),
        }
        std::process::exit(1);
    }

    let source = card_source_init();
    let storage = card_storage_init().await;
    let image_store = image_store_init();
    let emoji_store = emoji_store_init();

    match cli.command {
        Command::Spoilers => spoilers::sync(source, storage, image_store, emoji_store).await,
        Command::Bulk => bulk::sync(source, storage, image_store, emoji_store).await,
        Command::MigrateImages { .. } => unreachable!("handled before adapter init"),
    }
}

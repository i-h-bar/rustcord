mod adapters;
mod domain;
mod ports;

use crate::adapters::services::spoiler_sender_init;
use adapters::services::card_storage_init;
use adapters::services::init_image_store;
use domain::notify;
use std::time::Duration;

/// Round 1: `notifier` is a plain always-on process that polls on a fixed
/// interval, rather than a KEDA-scaled Job triggered by a Postgres watch
/// query. Simpler to deploy (no KEDA cluster dependency) at the cost of an
/// idle pod between polls — revisit scale-to-zero via KEDA later if that
/// idle cost or poll latency actually matters.
const POLL_INTERVAL: Duration = Duration::from_mins(1);

#[tokio::main]
async fn main() {
    env_logger::init();

    let storage = card_storage_init().await;
    let images = init_image_store();
    let sender = spoiler_sender_init();

    loop {
        notify::run(&storage, &images, &sender).await;
        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

mod adapters;
mod domain;
mod ports;

use adapters::services::card_storage_init;
use adapters::webhook::ReqwestWebhookSender;
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
    let sender = ReqwestWebhookSender::new();

    loop {
        domain::notify::run(&storage, &sender).await;
        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

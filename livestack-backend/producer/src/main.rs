use std::{sync::Arc, time::Duration};

use messaging::config::StreamService;
use std::env;
use store::Store;
use tokio_cron_scheduler::{Job, JobScheduler};

pub mod util;

const DEFAULT_REDIS_URL: &str = "redis://127.0.0.1/";
const PRODUCE_INTERVAL_SECONDS: u64 = 180;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| DEFAULT_REDIS_URL.to_string());
    // A pool, not a single long-lived connection: every tick checks a
    // connection out fresh, so a Postgres restart costs one failed cycle
    // instead of wedging the producer forever. Before this, a dropped
    // connection meant "no connection to the server" every 3 minutes until
    // someone noticed and restarted the process — with no checks queued in
    // the meantime, i.e. monitoring silently stopped.
    let pool = Store::worker_pool();
    let stream = Arc::new(StreamService::new(&redis_url)?);

    let sched = JobScheduler::new().await?;

    let job_pool = pool.clone();
    let job_stream = Arc::clone(&stream);

    let job = Job::new_repeated(
        Duration::from_secs(PRODUCE_INTERVAL_SECONDS),
        move |uuid, _lock| {
            println!("producer job {uuid} started");

            let websites = Store::from_pool(&job_pool)
                .map_err(|err| err.to_string())
                .and_then(|mut store| store.get_all_websites().map_err(|err| err.to_string()));

            match websites {
                Ok(websites) if websites.is_empty() => {
                    println!("producer found no websites to queue");
                }
                Ok(websites) => {
                    if let Err(err) = job_stream.add_records_batch(&websites) {
                        eprintln!("producer failed to queue website checks: {err}");
                    }
                }
                // Logged, never fatal: the next tick gets a fresh connection.
                Err(err) => {
                    eprintln!("producer failed to load websites: {err}");
                }
            }
        },
    )?;

    sched.add(job).await?;
    sched.start().await?;

    println!("producer running every {PRODUCE_INTERVAL_SECONDS} seconds");
    tokio::signal::ctrl_c().await?;

    Ok(())
}

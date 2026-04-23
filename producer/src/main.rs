use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use messaging::config::StreamService;
use store::Store;
use tokio_cron_scheduler::{Job, JobScheduler};

pub mod util;

const REDIS_URL: &str = "redis://127.0.0.1/";
const PRODUCE_INTERVAL_SECONDS: u64 = 180;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = Arc::new(Mutex::new(Store::default()?));
    let stream = Arc::new(StreamService::new(REDIS_URL)?);

    let sched = JobScheduler::new().await?;

    let job_store = Arc::clone(&store);
    let job_stream = Arc::clone(&stream);

    let job = Job::new_repeated(
        Duration::from_secs(PRODUCE_INTERVAL_SECONDS),
        move |uuid, _lock| {
            println!("producer job {uuid} started");

            let websites = {
                let mut store = job_store.lock().unwrap();
                store.get_all_websites()
            };

            match websites {
                Ok(websites) if websites.is_empty() => {
                    println!("producer found no websites to queue");
                }
                Ok(websites) => {
                    if let Err(err) = job_stream.add_records_batch(&websites) {
                        eprintln!("producer failed to queue website checks: {err}");
                    }
                }
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

use std::{
 sync::{Arc, Mutex}, time::Duration
};

use messaging::config::StreamService;
use store::{Store, models::website::Website};
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};
pub mod util;
const _CHUNK_LENGTH : u8 = 50;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), JobSchedulerError> {
    let arched_store = Arc::new(Mutex::new(Store::default().unwrap()));
    let stream = StreamService::new("redis://127.0.0.1/").unwrap();

    let sched = JobScheduler::new().await?;

    let mut jj = Job::new_repeated(Duration::from_secs(10), move |uuid, _l| {
        println!("I run repeatedly every 3 minute  {}", uuid);



        let websites : Vec<Website> = {
            let mut store = arched_store.lock().unwrap();
            store.get_all_websites()
        }.unwrap();

        stream.add_records();



        println!("{:?}" , websites);

        // should i write business logic
    })?;


    jj.on_start_notification_add(&sched, Box::new(|job_id, notification_id, type_of_notification| {
        Box::pin(async move {
            println!("Job {:?} was started, notification {:?} ran ({:?})", job_id, notification_id, type_of_notification);
        })
    })).await?;





    jj.on_stop_notification_add(&sched, Box::new(|job_id, notification_id, type_of_notification| {
        Box::pin(async move {
            println!("Job {:?} was completed, notification {:?} ran ({:?})", job_id, notification_id, type_of_notification);
        })
    })).await?;

    sched.add(jj).await?;
    sched.start().await?;
    tokio::time::sleep(Duration::from_secs(100)).await;



    Ok(())


}

use tokio_cron_scheduler::{JobScheduler, JobSchedulerError};

#[derive(thiserror::Error, Debug)]
pub enum CronJobError {
    #[error(transparent)]
    CronScheduler(#[from] JobSchedulerError),
}

pub async fn start_jobs() -> Result<(), CronJobError> {
    let scheduler = JobScheduler::new().await?;

    scheduler.shutdown_on_ctrl_c();
    scheduler.start().await?;
    Ok(())
}
async fn morning_greeting() {
    todo!()
}

async fn night_greeting() {
    todo!()
}

async fn weather_check() {
    todo!()
}

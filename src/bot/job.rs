use teloxide::Bot;
use tokio_cron_scheduler::JobScheduler;

use crate::types::MyResult;
pub async fn start_jobs(bot: &Bot) -> MyResult<JobScheduler> {
    let mut sched = JobScheduler::new().await?;

    //*
    // query from database
    // */
    sched.shutdown_on_ctrl_c();
    sched.start().await?;
    Ok(sched)
}

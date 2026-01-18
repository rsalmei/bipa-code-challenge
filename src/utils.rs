use anyhow::Result;
use std::time::Duration;

/// Spawns a periodic task that runs the given task at the specified interval.
pub async fn periodic_task<F>(interval: Duration, task: impl Fn() -> F)
where
    F: Future<Output = Result<()>> + Send + 'static,
{
    // by making the loop outside the task itself, we ensure that any errors won't ever break future
    // executions.
    loop {
        if let Err(err) = task().await {
            eprintln!("error updating nodes connectivity db: {err}");
            // we could use exponential backoff here, to let the external service recover
            // in case of persistent errors, but for now we just retry at the regular interval.
        }

        // we could use a tokio interval here to guarantee precise timing, but that could lead to
        // overlapping executions if the task takes longer than the interval, and put more strain
        // on external services in case of persistent errors.
        tokio::time::sleep(interval).await;
    }
}

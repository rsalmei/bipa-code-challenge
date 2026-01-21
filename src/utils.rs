use anyhow::Result;
use std::time::Duration;

/// Spawns a periodic task that runs the given task at the specified interval.
pub async fn periodic_task<F>(period: Duration, task: impl Fn() -> F)
where
    F: Future<Output = Result<()>> + Send + 'static,
{
    // we run an infinite loop here to execute the task periodically.
    // by making the loop outside the periodic task itself, we ensure that any errors won't ever
    // break future executions; they will be logged and the task retried after the interval.
    loop {
        if let Err(err) = task().await {
            eprintln!("periodic task error: {err}");
            // we could use exponential backoff here, to let the external service recover
            // in case of persistent errors, but for now we just retry at the regular interval.
        }

        // we could use a tokio interval here to guarantee precise timing, but that could lead to
        // overlapping executions if the task takes longer than the interval, and put more strain
        // on external services in case of persistent errors.
        tokio::time::sleep(period).await;
    }
}

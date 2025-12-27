# `tokio-warden-restartable`

A utility for creating auto-restarting tokio tasks.

This crate provides [`Restartable`], a future that wraps a spawned tokio task
and automatically restarts it whenever it completes or is aborted. This is useful
for long-running background tasks that should be resilient to failures.

## Example

```no_run
use std::time::Duration;
use tokio_warden_restartable::restartable;

#[tokio::main]
async fn main() {
    // Create a restartable task that automatically restarts on completion
    let restartable = restartable(|prev| {
        if let Some(result) = prev {
            println!("Previous task ended with: {:?}", result);
        }
        tokio::spawn(async {
            println!("Task started");
            tokio::time::sleep(Duration::from_secs(5)).await;
            println!("Task completed");
        })
    });

    // Get a handle to manually trigger restarts
    let restart_handle = restartable.restart_handle();

    // Spawn the restartable as a background task
    let handle = tokio::spawn(restartable);

    // Trigger a restart from anywhere
    tokio::time::sleep(Duration::from_secs(1)).await;
    restart_handle.restart();

    // The restartable runs forever until its outer handle is aborted
    handle.abort();
}
```


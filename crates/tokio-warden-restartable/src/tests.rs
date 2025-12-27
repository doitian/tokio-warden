use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

use super::*;

#[tokio::test]
async fn it_works_for_single_thread() {
    let restartable =
        restartable(|_| tokio::spawn(async { tokio::time::sleep(Duration::from_secs(1)).await }));
    let restartable_handle = tokio::spawn(restartable);
    restartable_handle.abort();
    let result = restartable_handle.await;
    assert!(result.is_err(), "restartable can only exit wit JoinError");
}

async fn abort_inner_task_triggers_restart_common() {
    let restart_count = Arc::new(AtomicUsize::new(0));
    let restart_count_clone = Arc::clone(&restart_count);

    let restartable = restartable(move |_prev| {
        restart_count_clone.fetch_add(1, Ordering::SeqCst);
        tokio::spawn(async {
            tokio::time::sleep(Duration::from_secs(10)).await;
        })
    });

    // Get the handle before spawning
    let restart_handle = restartable.restart_handle();
    let outer_handle = tokio::spawn(restartable);

    // Wait a bit for the first spawn
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(restart_count.load(Ordering::SeqCst), 1);

    // Abort the inner task - should trigger a restart
    restart_handle.restart();

    // Wait for restart to happen
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(restart_count.load(Ordering::SeqCst), 2);

    // Clean up
    outer_handle.abort();
}

#[tokio::test]
async fn abort_inner_task_triggers_restart_st() {
    abort_inner_task_triggers_restart_common().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn abort_inner_task_triggers_restart_mt() {
    abort_inner_task_triggers_restart_common().await;
}

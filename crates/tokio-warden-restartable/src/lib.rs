#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, RwLock},
    task::{Context, Poll},
};

use tokio::task::{AbortHandle, JoinError, JoinHandle};

/// A handle that can be used to restart the inner task of a [`Restartable`].
#[derive(Clone)]
pub struct RestartHandle {
    current_abort: Arc<RwLock<AbortHandle>>,
}

impl RestartHandle {
    /// Aborts the current running inner task and restarts a new one.
    pub fn restart(&self) {
        self.current_abort
            .read()
            .expect("failed to obtain read lock on abort handle")
            .abort();
    }
}

pub struct Restartable<F, T> {
    spawn: F,
    current: JoinHandle<T>,
    current_abort: Arc<RwLock<AbortHandle>>,
}

impl<F, T> Restartable<F, T> {
    /// Returns a handle that can be used to restart the inner task.
    pub fn restart_handle(&self) -> RestartHandle {
        RestartHandle {
            current_abort: Arc::clone(&self.current_abort),
        }
    }
}

impl<F, T> Future for Restartable<F, T>
where
    F: FnMut(Option<Result<T, JoinError>>) -> JoinHandle<T> + Unpin,
{
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        if let Poll::Ready(output) = Pin::new(&mut this.current).poll(cx) {
            this.current = (this.spawn)(Some(output));
            // Update the abort handle for the new task
            *this
                .current_abort
                .write()
                .expect("failed to obtain write lock on abort handle") =
                this.current.abort_handle();
            // Yield on restart
            cx.waker().wake_by_ref();
        }
        Poll::Pending
    }
}

/// Creates an auto restarting task.
///
/// The inner task is first created by `spawn` with `None`. Once the inner task
/// aborts or exits, a new inner task is created by calling `spawn` with the
/// previous result.
pub fn restartable<F, T>(mut spawn: F) -> Restartable<F, T>
where
    F: FnMut(Option<Result<T, JoinError>>) -> JoinHandle<T>,
{
    let current = spawn(None);
    let current_abort = Arc::new(RwLock::new(current.abort_handle()));
    Restartable {
        spawn,
        current,
        current_abort,
    }
}

#[cfg(test)]
mod tests;

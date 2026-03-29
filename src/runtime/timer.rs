use anyhow::{Context as AnyhowContext, Result};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimerResponse {
    pub request_id: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimerError {
    pub request_id: u64,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimerCompletion {
    Response(TimerResponse),
    #[allow(dead_code)]
    Error(TimerError),
}

impl TimerCompletion {
    pub fn request_id(&self) -> u64 {
        match self {
            Self::Response(response) => response.request_id,
            Self::Error(error) => error.request_id,
        }
    }
}

#[derive(Debug)]
pub struct TimerTransport {
    runtime: tokio::runtime::Runtime,
    completions_tx: Sender<TimerCompletion>,
    completions_rx: Receiver<TimerCompletion>,
    next_request_id: AtomicU64,
}

impl TimerTransport {
    pub fn new() -> Result<Self> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("rustyjs-timer")
            .build()
            .context("failed to build timer runtime")?;
        let (completions_tx, completions_rx) = mpsc::channel();

        Ok(Self {
            runtime,
            completions_tx,
            completions_rx,
            next_request_id: AtomicU64::new(1),
        })
    }

    pub fn submit(&self, delay: Duration) -> Result<u64> {
        let request_id = self.next_request_id.fetch_add(1, Ordering::Relaxed);
        let completions_tx = self.completions_tx.clone();

        self.runtime.spawn(async move {
            tokio::time::sleep(delay).await;
            let _ = completions_tx.send(TimerCompletion::Response(TimerResponse { request_id }));
        });

        Ok(request_id)
    }

    pub fn try_recv_completion(&self) -> Option<TimerCompletion> {
        match self.completions_rx.try_recv() {
            Ok(completion) => Some(completion),
            Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => None,
        }
    }

    pub fn drain_completions(&self) -> Vec<TimerCompletion> {
        let mut completions = Vec::new();

        while let Some(completion) = self.try_recv_completion() {
            completions.push(completion);
        }

        completions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timer_transport_completes_after_delay() {
        let transport = TimerTransport::new().unwrap();
        let request_id = transport.submit(Duration::from_millis(1)).unwrap();

        for _ in 0..200 {
            if let Some(completion) = transport.drain_completions().into_iter().next() {
                assert_eq!(completion.request_id(), request_id);
                return;
            }

            std::thread::sleep(Duration::from_millis(5));
        }

        panic!("timed out waiting for timer completion");
    }
}

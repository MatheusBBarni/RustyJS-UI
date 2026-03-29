use anyhow::{Context as AnyhowContext, Result};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageOperation {
    Get,
    Set,
    Remove,
    Clear,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageRequest {
    pub key: Option<String>,
    pub value: Option<String>,
    pub operation: StorageOperation,
}

impl StorageRequest {
    pub fn get(key: impl Into<String>) -> Self {
        Self {
            key: Some(key.into()),
            value: None,
            operation: StorageOperation::Get,
        }
    }

    pub fn set(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: Some(key.into()),
            value: Some(value.into()),
            operation: StorageOperation::Set,
        }
    }

    pub fn remove(key: impl Into<String>) -> Self {
        Self {
            key: Some(key.into()),
            value: None,
            operation: StorageOperation::Remove,
        }
    }

    pub fn clear() -> Self {
        Self {
            key: None,
            value: None,
            operation: StorageOperation::Clear,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageResponse {
    pub request_id: u64,
    pub value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageError {
    pub request_id: u64,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageCompletion {
    Response(StorageResponse),
    Error(StorageError),
}

impl StorageCompletion {
    pub fn request_id(&self) -> u64 {
        match self {
            Self::Response(response) => response.request_id,
            Self::Error(error) => error.request_id,
        }
    }
}

#[derive(Debug)]
pub struct StorageTransport {
    runtime: tokio::runtime::Runtime,
    completions_tx: Sender<StorageCompletion>,
    completions_rx: Receiver<StorageCompletion>,
    next_request_id: AtomicU64,
    storage_file: PathBuf,
}

impl StorageTransport {
    #[allow(dead_code)]
    pub fn new() -> Result<Self> {
        let default_path = std::env::temp_dir().join("rustyjs-ui-storage.json");
        Self::new_with_path(default_path)
    }

    pub fn new_with_path(path: impl AsRef<Path>) -> Result<Self> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("rustyjs-storage")
            .build()
            .context("failed to build storage runtime")?;
        let (completions_tx, completions_rx) = mpsc::channel();

        Ok(Self {
            runtime,
            completions_tx,
            completions_rx,
            next_request_id: AtomicU64::new(1),
            storage_file: path.as_ref().to_path_buf(),
        })
    }

    pub fn submit(&self, request: StorageRequest) -> Result<u64> {
        let request_id = self.next_request_id.fetch_add(1, Ordering::Relaxed);
        let storage_file = self.storage_file.clone();
        let completions_tx = self.completions_tx.clone();

        self.runtime.spawn(async move {
            let completion = tokio::task::spawn_blocking(move || {
                execute_request(storage_file.as_path(), request_id, request)
            })
            .await
            .unwrap_or_else(|error| {
                StorageCompletion::Error(StorageError {
                    request_id,
                    message: error.to_string(),
                })
            });

            let _ = completions_tx.send(completion);
        });

        Ok(request_id)
    }

    pub fn try_recv_completion(&self) -> Option<StorageCompletion> {
        match self.completions_rx.try_recv() {
            Ok(completion) => Some(completion),
            Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => None,
        }
    }

    pub fn drain_completions(&self) -> Vec<StorageCompletion> {
        let mut completions = Vec::new();

        while let Some(completion) = self.try_recv_completion() {
            completions.push(completion);
        }

        completions
    }
}

fn execute_request(
    storage_file: &Path,
    request_id: u64,
    request: StorageRequest,
) -> StorageCompletion {
    match run_request(storage_file, request) {
        Ok(value) => StorageCompletion::Response(StorageResponse { request_id, value }),
        Err(error) => StorageCompletion::Error(StorageError {
            request_id,
            message: error.to_string(),
        }),
    }
}

fn run_request(storage_file: &Path, request: StorageRequest) -> Result<Option<String>> {
    let mut data = read_storage_map(storage_file).with_context(|| {
        format!(
            "failed to load storage file `{}`",
            storage_file.display()
        )
    })?;

    let response = match request.operation {
        StorageOperation::Get => {
            let key = request
                .key
                .context("storage get requests require a key")?;
            data.get(&key).cloned()
        }
        StorageOperation::Set => {
            let key = request
                .key
                .context("storage set requests require a key")?;
            let value = request
                .value
                .context("storage set requests require a value")?;
            data.insert(key, value.clone());
            write_storage_map(storage_file, &data)?;
            Some(value)
        }
        StorageOperation::Remove => {
            let key = request
                .key
                .context("storage remove requests require a key")?;
            let removed = data.remove(&key);
            write_storage_map(storage_file, &data)?;
            removed
        }
        StorageOperation::Clear => {
            data.clear();
            write_storage_map(storage_file, &data)?;
            None
        }
    };

    Ok(response)
}

fn read_storage_map(storage_file: &Path) -> Result<BTreeMap<String, String>> {
    if !storage_file.exists() {
        return Ok(BTreeMap::new());
    }

    let raw = fs::read_to_string(storage_file)?;
    if raw.trim().is_empty() {
        return Ok(BTreeMap::new());
    }

    serde_json::from_str(&raw).context("storage file is not valid JSON")
}

fn write_storage_map(storage_file: &Path, data: &BTreeMap<String, String>) -> Result<()> {
    if let Some(parent) = storage_file.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "failed to create storage directory `{}`",
                parent.display()
            )
        })?;
    }

    let raw = serde_json::to_string_pretty(data).context("failed to serialize storage map")?;
    fs::write(storage_file, raw).with_context(|| {
        format!(
            "failed to write storage file `{}`",
            storage_file.display()
        )
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_storage_path(name: &str) -> PathBuf {
        let unique = format!(
            "rustyjs-ui-storage-{name}-{}.json",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );

        std::env::temp_dir().join(unique)
    }

    #[test]
    fn storage_request_builders_capture_expected_state() {
        let get = StorageRequest::get("theme");
        let set = StorageRequest::set("theme", "dark");
        let remove = StorageRequest::remove("theme");
        let clear = StorageRequest::clear();

        assert_eq!(get.operation, StorageOperation::Get);
        assert_eq!(set.operation, StorageOperation::Set);
        assert_eq!(remove.operation, StorageOperation::Remove);
        assert_eq!(clear.operation, StorageOperation::Clear);
    }

    #[test]
    fn storage_transport_persists_values_on_disk() {
        let storage_file = unique_storage_path("persist");
        let transport = StorageTransport::new_with_path(&storage_file).unwrap();

        let set_request_id = transport.submit(StorageRequest::set("theme", "dark")).unwrap();
        for _ in 0..200 {
            if transport.drain_completions().into_iter().any(|completion| {
                matches!(
                    completion,
                    StorageCompletion::Response(StorageResponse {
                        request_id,
                        value: Some(value)
                    }) if request_id == set_request_id && value == "dark"
                )
            }) {
                break;
            }

            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        let get_request_id = transport.submit(StorageRequest::get("theme")).unwrap();
        let mut completions = Vec::new();
        for _ in 0..200 {
            completions.extend(transport.drain_completions());
            if !completions.is_empty() {
                break;
            }

            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        assert!(completions
            .iter()
            .any(|completion| matches!(
                completion,
                StorageCompletion::Response(StorageResponse {
                    request_id,
                    value: Some(value)
                }) if *request_id == get_request_id && value == "dark"
            )));

        let _ = fs::remove_file(storage_file);
    }
}

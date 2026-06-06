//! File watching store support.

use std::{
    io,
    path::PathBuf,
    sync::{
        mpsc::{self, RecvTimeoutError, Sender},
        Arc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::watch;

use super::{load_from_root, LoadError, Snippet};

const DEBOUNCE_WINDOW: Duration = Duration::from_millis(200);

#[derive(Debug, Clone)]
pub struct SnapshotInner {
    pub revision: u64,
    pub snippets: Vec<Snippet>,
    pub errors: Vec<LoadError>,
}

pub struct Store {
    sender: watch::Sender<Arc<SnapshotInner>>,
    trigger_tx: Sender<Trigger>,
    worker: Option<JoinHandle<()>>,
}

enum Trigger {
    Filesystem,
    ReloadNow,
    Shutdown,
}

impl Store {
    pub fn spawn(root: PathBuf) -> notify::Result<Self> {
        let initial_snapshot = Arc::new(load_snapshot(&root, 0));
        let (sender, _) = watch::channel(initial_snapshot);
        let (trigger_tx, trigger_rx) = mpsc::channel();

        let watcher_trigger = trigger_tx.clone();
        let mut watcher: RecommendedWatcher = notify::recommended_watcher(move |event| {
            let _ = event;
            let _ = watcher_trigger.send(Trigger::Filesystem);
        })?;
        watcher.watch(&root, RecursiveMode::Recursive)?;

        let worker_sender = sender.clone();
        let worker = thread::spawn(move || {
            let _watcher = watcher;
            let mut revision = 0;

            while let Ok(first_trigger) = trigger_rx.recv() {
                match first_trigger {
                    Trigger::Shutdown => break,
                    Trigger::Filesystem | Trigger::ReloadNow => {
                        if !wait_for_quiet_period(&trigger_rx) {
                            break;
                        }

                        revision += 1;
                        let snapshot = Arc::new(load_snapshot(&root, revision));
                        let _ = worker_sender.send(snapshot);
                    }
                }
            }
        });

        Ok(Self {
            sender,
            trigger_tx,
            worker: Some(worker),
        })
    }

    pub fn subscribe(&self) -> watch::Receiver<Arc<SnapshotInner>> {
        self.sender.subscribe()
    }

    pub fn reload_now(&self) {
        let _ = self.trigger_tx.send(Trigger::ReloadNow);
    }
}

pub fn watch_root(root: PathBuf) -> notify::Result<Store> {
    Store::spawn(root)
}

impl Drop for Store {
    fn drop(&mut self) {
        let _ = self.trigger_tx.send(Trigger::Shutdown);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn wait_for_quiet_period(trigger_rx: &mpsc::Receiver<Trigger>) -> bool {
    loop {
        match trigger_rx.recv_timeout(DEBOUNCE_WINDOW) {
            Ok(Trigger::Shutdown) => return false,
            Ok(Trigger::Filesystem | Trigger::ReloadNow) => continue,
            Err(RecvTimeoutError::Timeout) => return true,
            Err(RecvTimeoutError::Disconnected) => return false,
        }
    }
}

fn load_snapshot(root: &std::path::Path, revision: u64) -> SnapshotInner {
    match load_from_root(root) {
        Ok(result) => SnapshotInner {
            revision,
            snippets: result.snippets,
            errors: result.errors,
        },
        Err(error) => SnapshotInner {
            revision,
            snippets: Vec::new(),
            errors: vec![LoadError::Io {
                path: root.to_path_buf(),
                message: io_error_message(error),
            }],
        },
    }
}

fn io_error_message(error: io::Error) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{Duration, Instant},
    };

    use tempfile::TempDir;

    use super::Store;

    fn write_yaml(root: &TempDir, relative_path: &str, contents: &str) {
        let path = root.path().join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, contents).unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn publishes_snapshot_after_file_create_within_500ms() {
        let root = TempDir::new().unwrap();
        let store = Store::spawn(root.path().to_path_buf()).unwrap();
        let mut rx = store.subscribe();
        let initial_revision = rx.borrow().revision;

        write_yaml(
            &root,
            "created.yaml",
            r#"
version: 1
snippets:
  - trigger: ;sig
    replace: hello
"#,
        );

        let changed = tokio::time::timeout(Duration::from_millis(500), async {
            loop {
                rx.changed().await.unwrap();
                if rx.borrow().revision > initial_revision {
                    break rx.borrow().clone();
                }
            }
        })
        .await
        .expect("timed out waiting for snapshot");

        assert_eq!(changed.snippets.len(), 1);
        assert!(changed.errors.is_empty());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn watcher_latency_p95_stays_below_250ms() {
        let root = TempDir::new().unwrap();
        let store = Store::spawn(root.path().to_path_buf()).unwrap();
        let mut rx = store.subscribe();
        let file_path = root.path().join("latency.yaml");
        let mut timings_ms = Vec::new();

        for iteration in 0..20 {
            let revision = rx.borrow().revision;
            let started = Instant::now();
            fs::write(
                &file_path,
                format!(
                    "version: 1\nsnippets:\n  - trigger: ;sig\n    replace: run-{iteration}\n"
                ),
            )
            .unwrap();

            tokio::time::timeout(Duration::from_millis(500), async {
                loop {
                    rx.changed().await.unwrap();
                    if rx.borrow().revision > revision {
                        break;
                    }
                }
            })
            .await
            .expect("timed out waiting for snapshot");

            timings_ms.push(started.elapsed().as_millis() as u64);
        }

        timings_ms.sort_unstable();
        println!("watcher latencies ms: {:?}", timings_ms);
        let p95_index = ((timings_ms.len() * 95).div_ceil(100)).saturating_sub(1);
        let p95 = timings_ms[p95_index];
        assert!(p95 < 250, "expected p95 < 250ms, got {p95}ms");
    }
}

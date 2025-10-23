use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
pub enum AppEvent {
    //    FileChange(PathBuf),
    FileChange,
}
pub struct FileWatcher {
    _thread: std::thread::JoinHandle<()>,
    _file_name: String,
}
impl FileWatcher {
    pub fn new(file_path: PathBuf, tx: Sender<AppEvent>) -> Self {
        let _file_name = file_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let (notify_tx, notify_rx) = std::sync::mpsc::channel();
        let value = _file_name.clone();
        let thread = std::thread::spawn(move || {
            let mut watcher = RecommendedWatcher::new(notify_tx, Config::default()).unwrap();
            watcher
                .watch(Path::new(&file_path), RecursiveMode::Recursive)
                .unwrap();
            while let Ok(Ok(event)) = notify_rx.recv() {
                if let notify::EventKind::Modify(_) = event.kind {
                    for p in event.paths {
                        if p.file_name() == Some(value.as_ref()) {
                            //                            let _ = tx.send(AppEvent::FileChange(file_path.clone()));
                            let _ = tx.send(AppEvent::FileChange);
                        }
                    }
                }
            }
        });
        Self {
            _thread: thread,
            _file_name,
        }
    }
}

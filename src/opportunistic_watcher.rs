use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::time::Duration;

use errors::{Result, ResultExt};

use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use notify::DebouncedEvent::{Create, Rename};

pub struct OpportunisticWatcher {
    watcher: RecommendedWatcher,
    watches: HashMap<PathBuf, RecursiveMode>,
}

impl OpportunisticWatcher {
    pub fn new(tx: Sender<DebouncedEvent>, delay: Duration) -> Result<Self> {
        let watcher = RecommendedWatcher::new(tx, delay).unwrap();
        let opportunistic_watcher = OpportunisticWatcher {
            watcher,
            watches: HashMap::new(),
        };
        Ok(opportunistic_watcher)
    }

    pub fn watch<P: AsRef<Path>>(&mut self, path: P, recursive_mode: RecursiveMode) -> Result<()> {
        self.watches.insert(path.as_ref().to_path_buf(), recursive_mode);
        self.watcher.watch(path.as_ref().clone(), recursive_mode)
            .chain_err(|| format!("Failed to watch `{}`", path.as_ref().display()))
    }

    pub fn unwatch<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        self.watches.remove(path.as_ref());
        self.watcher.unwatch(path.as_ref().clone())
            .chain_err(|| format!("Failed to unwatch `{}`", path.as_ref().display()))
    }

    pub fn watches(&self) -> &HashMap<PathBuf, RecursiveMode> {
        &self.watches
    }

    pub fn update(&mut self, event: &DebouncedEvent) -> Result<()> {
        match &event {
            Create(path) |
            Rename(_, path) => {
                match self.watches.get(path) {
                    Some(recursive_mode) => {
                        self.watcher.watch(&path, *recursive_mode)
                            .chain_err(|| format!("Failed to watch `{}`", path.display()))
                    },
                    None => Ok(()),
                }
            },
            _ => Ok(()),
        }
    }
}

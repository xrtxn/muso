use log::debug;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

use notify::event::EventKind;
use notify::RecursiveMode;
use notify::Watcher as _;
use notify_debouncer_full::{new_debouncer, DebounceEventResult};

use crate::config::Config;
use crate::sorting::{sort_file, sort_folder, Options};
use crate::{Error, Result};

#[derive(Debug, Clone)]
pub struct Watcher {
    config: Config,
    roots: HashMap<PathBuf, String>,
    ignore: HashSet<PathBuf>,
}

impl Watcher {
    pub fn new(config: Config) -> Self {
        let mut roots = HashMap::new();

        for (name, library) in &config.libraries {
            for folder in &library.folders {
                roots.insert(folder.to_owned(), name.to_owned());
            }
        }

        Self {
            config,
            roots,
            ignore: HashSet::new(),
        }
    }

    pub fn watch(self) -> Result<()> {
        if self.config.libraries.is_empty() {
            log::info!("No directories to watch!");
            return Ok(());
        }

        let (tx, rx) = mpsc::channel();
        let delay = Duration::from_secs(self.config.watch.every.unwrap_or(1));
        let mut debouncer = new_debouncer(delay, None, tx)?;

        for root in self.roots.keys() {
            debouncer.watcher().watch(root, RecursiveMode::Recursive)?;
        }

        log::info!("Watching libraries");
        self.watchloop(rx)
    }

    fn watchloop(mut self, rx: Receiver<DebounceEventResult>) -> Result<()> {
        loop {
            for result in &rx {
                match result {
                    Err(err) => {
                        log::error!("{:?}", err);
                        continue;
                    }

                    Ok(event) => {
                        for ev in event {
                            debug!("{:?}", ev);
                            match ev.event.kind {
                                EventKind::Other => {
                                    continue;
                                }

                                EventKind::Create(_) => {
                                    for path in &ev.paths {
                                        if self.is_ignored(path) {
                                            self.ignore.remove(path);
                                            continue;
                                        }
                                        match self.move_files(path) {
                                            Ok(_) => {}
                                            Err(_) => continue,
                                        };
                                    }
                                }
                                EventKind::Modify(notify::event::ModifyKind::Name(
                                    notify::event::RenameMode::Both,
                                )) => {
                                    for path in ev.paths.iter().skip(1).step_by(2) {
                                        if self.is_ignored(path) {
                                            self.ignore.remove(path);
                                            continue;
                                        }
                                        match self.move_files(path) {
                                            Ok(_) => {}
                                            Err(_) => continue,
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }

    fn ignore_path<P, R>(&mut self, path: P, root: R) -> Result<()>
    where
        P: AsRef<Path>,
        R: AsRef<Path>,
    {
        let root = root.as_ref();
        let path = path.as_ref();

        let parent = path.parent().ok_or(Error::InvalidParent {
            child: path.to_string_lossy().into(),
        })?;

        //why is this necessary?
        if parent != root {
            self.ignore.insert(parent.to_path_buf());
        }

        self.ignore.insert(root.to_path_buf().join(path));

        Ok(())
    }

    fn is_ignored(&self, path: impl AsRef<Path>) -> bool {
        let path = path.as_ref();

        if path.is_file() {
            self.ignore.contains(path)
        } else {
            for ignored in &self.ignore {
                if !ignored.is_dir() {
                    continue;
                }

                if ignored.starts_with(path) {
                    return true;
                }
            }

            false
        }
    }

    fn root_for(&self, path: impl AsRef<Path>) -> Option<PathBuf> {
        let path = path.as_ref();
        for ancestor in path.ancestors() {
            if self.roots.contains_key(ancestor) {
                return Some(ancestor.to_path_buf());
            }
        }

        None
    }

    fn move_files(&mut self, path: &Path) -> Result<()> {
        if let Some(root) = self.root_for(path) {
            let library = &self.roots[&root];

            let options = Options {
                format: Cow::Borrowed(self.config.format_of(library).unwrap()),
                dryrun: false,
                recursive: true,
                exfat_compat: self.config.is_exfat_compat(library),
                remove_empty: true,
            };

            if path.is_dir() {
                match sort_folder(&root, path, &options) {
                    Ok(report) => {
                        log::info!(
                            "Done: {} successful out of {} ({} failed)",
                            report.success,
                            report.total,
                            report.total - report.success
                        );

                        for new_path in report.new_paths {
                            self.ignore_path(new_path, &root)?;
                        }
                        Ok(())
                    }

                    Err(e) => {
                        log::error!("{}", e);
                        Err(e)
                    }
                }
            } else {
                match sort_file(&root, path, &options) {
                    Ok(new_path) => {
                        log::info!("Done: 1 successful out of 1 (0 failed)");
                        self.ignore_path(new_path, root)?;
                        Ok(())
                    }

                    Err(e) => {
                        log::error!("{}", e);
                        Err(e)
                    }
                }
            }
        } else {
            Err(Error::InvalidRoot {
                path: path.to_string_lossy().to_string(),
            })
        }
    }
}

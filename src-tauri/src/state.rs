use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
};

use tauri::{AppHandle, Manager};
use tokio::sync::Semaphore;

use crate::models::{AppConfig, GenerationJob};

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub root: PathBuf,
    pub config_file: PathBuf,
    pub jobs_file: PathBuf,
    pub outputs: PathBuf,
    pub designs: PathBuf,
}

struct AppStateInner {
    app: AppHandle,
    config: RwLock<AppConfig>,
    jobs: RwLock<HashMap<String, GenerationJob>>,
    paths: AppPaths,
    generation_gate: Arc<Semaphore>,
    automation_started: AtomicBool,
}

#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

impl AppState {
    pub fn initialize(app: &AppHandle) -> Result<Self, String> {
        let root = app
            .path()
            .app_data_dir()
            .map_err(|error| format!("Unable to resolve app data directory: {error}"))?;
        let outputs = root.join("outputs");
        let designs = root.join("designs");

        fs::create_dir_all(&outputs)
            .map_err(|error| format!("Unable to create output directory: {error}"))?;
        fs::create_dir_all(&designs)
            .map_err(|error| format!("Unable to create design directory: {error}"))?;

        let paths = AppPaths {
            config_file: root.join("config.json"),
            jobs_file: root.join("jobs.json"),
            root,
            outputs,
            designs,
        };

        let config = read_json::<AppConfig>(&paths.config_file).unwrap_or_default();
        let jobs = read_json::<Vec<GenerationJob>>(&paths.jobs_file)
            .unwrap_or_default()
            .into_iter()
            .map(|job| (job.id.clone(), job))
            .collect();

        let state = Self {
            inner: Arc::new(AppStateInner {
                app: app.clone(),
                config: RwLock::new(config),
                jobs: RwLock::new(jobs),
                paths,
                generation_gate: Arc::new(Semaphore::new(1)),
                automation_started: AtomicBool::new(false),
            }),
        };

        state.persist_config()?;
        state.persist_jobs()?;
        Ok(state)
    }

    pub fn app(&self) -> AppHandle {
        self.inner.app.clone()
    }

    pub fn paths(&self) -> AppPaths {
        self.inner.paths.clone()
    }

    pub fn config(&self) -> AppConfig {
        self.inner
            .config
            .read()
            .expect("config lock poisoned")
            .clone()
    }

    pub fn replace_config(&self, config: AppConfig) -> Result<AppConfig, String> {
        *self
            .inner
            .config
            .write()
            .map_err(|_| "Config lock was poisoned.".to_string())? = config.clone();
        self.persist_config()?;
        Ok(config)
    }

    pub fn list_jobs(&self) -> Vec<GenerationJob> {
        let mut jobs = self
            .inner
            .jobs
            .read()
            .expect("jobs lock poisoned")
            .values()
            .cloned()
            .collect::<Vec<_>>();
        jobs.sort_by(|left, right| right.created_at.cmp(&left.created_at));
        jobs
    }

    pub fn get_job(&self, id: &str) -> Option<GenerationJob> {
        self.inner
            .jobs
            .read()
            .expect("jobs lock poisoned")
            .get(id)
            .cloned()
    }

    pub fn insert_job(&self, job: GenerationJob) -> Result<(), String> {
        self.inner
            .jobs
            .write()
            .map_err(|_| "Jobs lock was poisoned.".to_string())?
            .insert(job.id.clone(), job);
        self.persist_jobs()
    }

    pub fn update_job<F>(&self, id: &str, update: F) -> Result<Option<GenerationJob>, String>
    where
        F: FnOnce(&mut GenerationJob),
    {
        let updated = {
            let mut jobs = self
                .inner
                .jobs
                .write()
                .map_err(|_| "Jobs lock was poisoned.".to_string())?;
            let Some(job) = jobs.get_mut(id) else {
                return Ok(None);
            };
            update(job);
            job.clone()
        };
        self.persist_jobs()?;
        Ok(Some(updated))
    }

    pub fn generation_gate(&self) -> Arc<Semaphore> {
        self.inner.generation_gate.clone()
    }

    pub fn mark_automation_started(&self) -> bool {
        self.inner
            .automation_started
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
    }

    pub fn reset_automation_started(&self) {
        self.inner.automation_started.store(false, Ordering::SeqCst);
    }

    pub fn is_path_inside_app_data(&self, path: &Path) -> bool {
        let Ok(candidate) = path.canonicalize() else {
            return false;
        };
        let Ok(root) = self.inner.paths.root.canonicalize() else {
            return false;
        };
        candidate.starts_with(root)
    }

    fn persist_config(&self) -> Result<(), String> {
        let config = self.config();
        write_json(&self.inner.paths.config_file, &config)
    }

    fn persist_jobs(&self) -> Result<(), String> {
        let jobs = self.list_jobs();
        write_json(&self.inner.paths.jobs_file, &jobs)
    }
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Option<T> {
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let content = serde_json::to_string_pretty(value)
        .map_err(|error| format!("Unable to serialize {}: {error}", path.display()))?;
    fs::write(path, content)
        .map_err(|error| format!("Unable to write {}: {error}", path.display()))
}

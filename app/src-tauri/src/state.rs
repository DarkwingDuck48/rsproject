use logic::SingleProjectContainer;
use std::{path::PathBuf, sync::Mutex};
use uuid::Uuid;

pub struct AppState {
    pub container: Mutex<SingleProjectContainer>,
    pub selected_project_id: Mutex<Option<Uuid>>,
    pub selected_task_id: Mutex<Option<Uuid>>,
    pub selected_resource_id: Mutex<Option<Uuid>>,
    pub critical_path: Mutex<Option<Vec<Uuid>>>,
    pub last_project_dir: Mutex<Option<PathBuf>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            container: Mutex::new(SingleProjectContainer::new()),
            selected_project_id: Mutex::new(None),
            selected_task_id: Mutex::new(None),
            selected_resource_id: Mutex::new(None),
            critical_path: Mutex::new(None),
            last_project_dir: Mutex::new(None),
        }
    }
    pub fn container(&self) -> std::sync::MutexGuard<'_, SingleProjectContainer> {
        self.container.lock().unwrap()
    }
}

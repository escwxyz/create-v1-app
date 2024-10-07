use crate::logger;
use std::fs;
use std::path::PathBuf;

pub enum CleanupTask {
    RemoveDirectory(PathBuf),
    RemoveFile(PathBuf),
    RemoveService {
        project_dir: PathBuf,
        service_name: String,
    },
}

pub struct CleanupManager {
    tasks: Vec<CleanupTask>,
}

impl CleanupManager {
    pub fn new() -> Self {
        CleanupManager { tasks: Vec::new() }
    }

    pub fn add_task(&mut self, task: CleanupTask) {
        self.tasks.push(task);
    }

    pub fn cleanup(&self) {
        logger::log_debug("Starting cleanup...");

        for task in &self.tasks {
            match task {
                CleanupTask::RemoveDirectory(path) => {
                    if path.exists() {
                        match fs::remove_dir_all(path) {
                            Ok(_) => {
                                logger::log_debug(&format!("Removed directory: {}", path.display()))
                            }
                            Err(e) => logger::log_error(&format!(
                                "Failed to remove directory {}: {}",
                                path.display(),
                                e
                            )),
                        }
                    }
                }
                CleanupTask::RemoveFile(path) => {
                    if path.exists() {
                        match fs::remove_file(path) {
                            Ok(_) => {
                                logger::log_debug(&format!("Removed file: {}", path.display()))
                            }
                            Err(e) => logger::log_error(&format!(
                                "Failed to remove file {}: {}",
                                path.display(),
                                e
                            )),
                        }
                    }
                }
                CleanupTask::RemoveService {
                    project_dir,
                    service_name,
                } => {
                    // Implement logic to remove a specific service from the project
                    // This might involve removing files, updating configuration, etc.
                    logger::log_debug(&format!(
                        "Removing service {} from {}",
                        service_name,
                        project_dir.display()
                    ));
                    // TODO: Implement service removal logic
                }
            }
        }

        logger::log_debug("Cleanup completed.");
    }
}

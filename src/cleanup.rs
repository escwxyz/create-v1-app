use crate::logger;
use anyhow::Result;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub enum CleanupTask {
    RemoveDirectory(PathBuf),
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
                CleanupTask::RemoveService {
                    project_dir,
                    service_name,
                } => {
                    logger::log_debug(&format!(
                        "Removing service {} from {}",
                        service_name,
                        project_dir.display()
                    ));

                    if let Err(e) = remove_service(project_dir, service_name) {
                        logger::log_error(&format!("Failed to remove service: {}", e));
                    }
                }
            }
        }

        logger::log_debug("Cleanup completed.");
    }
}

fn remove_service(project_dir: &Path, service_name: &str) -> Result<()> {
    logger::log_debug(&format!(
        "Removing service {} from {}",
        service_name,
        project_dir.display()
    ));

    let service_dir = project_dir.join("packages").join(service_name);
    if service_dir.exists() {
        fs::remove_dir_all(&service_dir)?;
        logger::log_debug(&format!(
            "Removed service directory: {}",
            service_dir.display()
        ));
    }

    update_root_package_json(project_dir, service_name)?;
    remove_service_references(project_dir, service_name)?;

    Ok(())
}

fn update_root_package_json(project_dir: &Path, service_name: &str) -> Result<()> {
    let package_json_path = project_dir.join("package.json");
    let content = fs::read_to_string(&package_json_path)?;
    let mut package_json: serde_json::Value = serde_json::from_str(&content)?;

    if let Some(workspaces) = package_json["workspaces"].as_array_mut() {
        let service_workspace = format!("packages/{}", service_name);
        workspaces.retain(|w| w != &service_workspace);
        let new_content = serde_json::to_string_pretty(&package_json)?;
        fs::write(package_json_path, new_content)?;
        logger::log_debug("Updated root package.json");
    }

    Ok(())
}

fn remove_service_references(project_dir: &Path, service_name: &str) -> Result<()> {
    let import_regex = Regex::new(&format!(
        r#"(?m)^\s*import.*from ["']@v1/{service_name}["'].*$"#
    ))?;
    let use_regex = Regex::new(&format!(r#"(?m)^\s*use.*@v1/{service_name}.*$"#))?;

    for entry in WalkDir::new(project_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file()
            && path.extension().map_or(false, |ext| {
                ext == "ts" || ext == "tsx" || ext == "js" || ext == "jsx"
            })
        {
            let content = fs::read_to_string(path)?;
            let new_content = import_regex.replace_all(&content, "").to_string();
            let new_content = use_regex.replace_all(&new_content, "").to_string();

            if new_content != content {
                fs::write(path, new_content)?;
                logger::log_debug(&format!("Updated file: {}", path.display()));
            }
        }
    }

    Ok(())
}

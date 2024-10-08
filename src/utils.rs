use std::{
    path::{Path, PathBuf},
    sync::Arc,
    thread,
    time::Duration,
};

use anyhow::{anyhow, Result};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use regex::Regex;

use crate::{
    logger::{log_debug, log_error},
    workspace::get_workspaces,
};

pub struct PackageJson {
    pub name: String,
    pub package_manager: String,
}

pub fn get_package_json(dir: Option<&Path>) -> Result<PackageJson> {
    let current_dir = dir.unwrap_or_else(|| Path::new("."));
    let package_json_path = current_dir.join("package.json");

    if !package_json_path.exists() {
        return Err(anyhow!("package.json not found in the current directory"));
    }

    let package_json = std::fs::read_to_string(package_json_path)?;
    let package_json: serde_json::Value = serde_json::from_str(&package_json)?;

    let name = package_json["name"]
        .as_str()
        .ok_or_else(|| anyhow!("name not found in package.json"))?;

    let package_manager = package_json["packageManager"]
        .as_str()
        .unwrap()
        .split('@')
        .next()
        .ok_or_else(|| anyhow!("packageManager not found in package.json"))?;

    Ok(PackageJson {
        name: name.to_string(),
        package_manager: package_manager.to_string(),
    })
}

pub fn select_package_manager() -> Result<String> {
    let package_managers = vec!["npm", "yarn", "pnpm", "bun"];
    let selection = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Select a package manager")
        .items(&package_managers)
        .default(0)
        .interact()
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    Ok(package_managers[selection].to_string())
}

pub fn confirm_package_manager(package_manager: Option<&str>) -> Result<String> {
    match package_manager {
        Some(pm) => Ok(pm.to_string()),
        None => select_package_manager(),
    }
}

#[allow(dead_code)]
pub fn install_all_dependencies(project_path: &Path, package_manager: &str) -> Result<()> {
    let multi_progress = Arc::new(MultiProgress::new());
    let style = ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
        .unwrap()
        .progress_chars("##-");

    let workspaces = get_workspaces(Path::new("templates"), project_path);
    let mut handles = vec![];

    for workspace in workspaces {
        let mp = multi_progress.clone();
        let pb = mp.add(ProgressBar::new(100));
        pb.set_style(style.clone());
        pb.set_message(format!("Installing {}", workspace.name));

        let package_manager = package_manager.to_string();
        let workspace_path = workspace.dest_path.to_path_buf();

        let handle = thread::spawn(move || {
            install_workspace_dependencies(&workspace_path, &package_manager, Some(pb))
        });

        handles.push(handle);
    }

    let results: Vec<Result<()>> = handles
        .into_iter()
        .map(|h| h.join().unwrap_or(Err(anyhow!("Thread panicked"))))
        .collect();

    for result in &results {
        if let Err(e) = result {
            eprintln!("Installation error: {}", e);
        }
    }

    if results.iter().any(|r| r.is_err()) {
        return Err(anyhow!("One or more installations failed"));
    }

    multi_progress.clear()?;

    Ok(())
}

pub fn install_workspace_dependencies(
    workspace_path: &Path,
    package_manager: &str,
    progress_bar: Option<ProgressBar>,
) -> Result<()> {
    log_debug(&format!(
        "Starting installation for workspace: {}",
        workspace_path.display()
    ));

    log_debug(&format!("Running command: {} install", package_manager));

    let pb = progress_bar.unwrap_or_else(|| {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap()
                .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ "),
        );
        pb.set_message(format!(
            "Installing dependencies for {}",
            workspace_path.display()
        ));
        pb.enable_steady_tick(Duration::from_millis(100));
        pb
    });

    let output = std::process::Command::new(package_manager)
        .arg("install")
        .current_dir(workspace_path)
        .output()?;

    log_debug(&format!(
        "Installation command completed for {}",
        workspace_path.display()
    ));

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let error_message = format!(
            "Installation failed in {}:\nStdout: {}\nStderr: {}",
            workspace_path.display(),
            stdout,
            stderr
        );
        log_error(&error_message);
        pb.finish_with_message("Installation failed");
        return Err(anyhow!(error_message));
    }

    let node_modules_path = workspace_path.join("node_modules");
    if !node_modules_path.exists() {
        let error_message = format!("node_modules not created in {}", workspace_path.display());
        log_error(&error_message);
        pb.finish_with_message("Installation failed");
        return Err(anyhow!(error_message));
    }

    pb.finish_with_message(format!("Installed {}", workspace_path.display()));

    Ok(())
}

pub fn is_valid_project_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(anyhow!("Project name cannot be empty"));
    }

    if !name.chars().next().unwrap().is_ascii_alphabetic() {
        return Err(anyhow!("Project name must start with a letter"));
    }

    let re = Regex::new(r"^[a-zA-Z][a-zA-Z0-9_-]*$").unwrap();
    if !re.is_match(name) {
        return Err(anyhow!(
            "Project name can only contain letters, numbers, dashes, and underscores"
        ));
    }

    if name.len() > 20 {
        return Err(anyhow!("Project name is too long (maximum 20 characters)"));
    }

    Ok(())
}

pub fn get_templates_path() -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    PathBuf::from(manifest_dir).join("templates") // templates/ inside the CLI project root
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_project_names() {
        assert!(is_valid_project_name("myproject").is_ok());
        assert!(is_valid_project_name("my-project").is_ok());
        assert!(is_valid_project_name("my_project").is_ok());
        assert!(is_valid_project_name("MyProject123").is_ok());
    }

    #[test]
    fn test_invalid_project_names() {
        assert!(is_valid_project_name("").is_err());
        assert!(is_valid_project_name("123project").is_err());
        assert!(is_valid_project_name("-project").is_err());
        assert!(is_valid_project_name("_project").is_err());
        assert!(is_valid_project_name("my project").is_err());
        assert!(is_valid_project_name("my@project").is_err());
        assert!(is_valid_project_name("MY_PROJECT!").is_err());
        assert!(is_valid_project_name("myproject12345678901234567890").is_err());
    }
}

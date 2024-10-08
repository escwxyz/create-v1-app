use anyhow::Result;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tera::Tera;
use walkdir::WalkDir;

use crate::logger::log_debug;
#[derive(Clone)]
pub struct Workspace {
    pub name: String,
    pub source_path: PathBuf,
    pub dest_path: PathBuf,
    pub is_root: bool,
}

pub fn process_workspace(
    workspace: &Workspace,
    tera: &Tera,
    context: &tera::Context,
    package_manager: &str,
) -> Result<()> {
    log_debug(&format!("Processing workspace: {}", workspace.name));

    let walker = if workspace.is_root {
        WalkDir::new(&workspace.source_path)
            .min_depth(1)
            .max_depth(1)
    } else {
        WalkDir::new(&workspace.source_path)
    };

    for entry in walker {
        let entry = entry.map_err(|e| anyhow::anyhow!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        if path.is_file() {
            let is_template = path.extension().map_or(false, |ext| ext == "tera");
            let relative_path = path
                .strip_prefix(&workspace.source_path)
                .map_err(|e| anyhow::anyhow!("Failed to strip prefix from path: {}", e))?;

            let file_name = relative_path
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Failed to convert path to string"))?;

            if is_template {
                process_template(workspace, tera, context, package_manager, path, file_name)?;
            } else {
                copy_non_template_file(workspace, path, file_name, package_manager)?;
            }
        }
    }
    Ok(())
}

fn process_template(
    workspace: &Workspace,
    tera: &Tera,
    context: &tera::Context,
    package_manager: &str,
    path: &Path,
    template_name: &str,
) -> Result<()> {
    log_debug(&format!("Processing template: {}", template_name));

    // Skip pnpm-workspace.yaml if package manager is not pnpm
    if template_name.ends_with("pnpm-workspace.yaml.tera") && package_manager != "pnpm" {
        log_debug(&format!(
            "Skipping pnpm-workspace.yaml for non-pnpm project"
        ));
        return Ok(());
    }

    // Handle package.json.*.tera files
    if template_name.starts_with("package.json.") {
        if !workspace.is_root {
            log_debug(&format!(
                "Skipping package.json template for non-root workspace"
            ));
            return Ok(()); // Skip for non-root workspaces
        }
        let pm_suffix = template_name
            .trim_end_matches(".tera")
            .split('.')
            .last()
            .unwrap_or("");
        if pm_suffix != package_manager && pm_suffix != "base" {
            log_debug(&format!(
                "Skipping non-matching package.json template: {}",
                template_name
            ));
            return Ok(());
        }
    }

    let rendered = tera
        .render(template_name, context)
        .map_err(|e| anyhow::anyhow!("Failed to render template {}: {}", template_name, e))?;

    // Skip empty templates (conditionally excluded)
    if rendered.trim().is_empty() {
        log_debug(&format!("Skipping empty template: {}", template_name));
        return Ok(());
    }

    let relative_dest_path = path
        .strip_prefix(&workspace.source_path)
        .map_err(|e| anyhow::anyhow!("Failed to strip prefix from destination path: {}", e))?;
    let mut dest_path = workspace
        .dest_path
        .join(relative_dest_path)
        .with_extension("");

    // Rename package.json.{pm}.tera to package.json for root workspace
    if workspace.is_root && template_name.starts_with("package.json.") {
        if template_name.ends_with("base.tera") {
            log_debug(&format!("Skipping base package.json template"));
            return Ok(());
        }
        dest_path = workspace.dest_path.join("package.json");
    }

    fs::create_dir_all(dest_path.parent().unwrap())?;
    fs::write(&dest_path, rendered)
        .map_err(|e| anyhow::anyhow!("Failed to write file {}: {}", dest_path.display(), e))?;

    log_debug(&format!(
        "Rendered template: {} -> {}",
        template_name,
        dest_path.display()
    ));
    Ok(())
}

fn copy_non_template_file(
    workspace: &Workspace,
    path: &Path,
    file_name: &str,
    package_manager: &str,
) -> Result<()> {
    // Skip package.json.*.tera files in root workspace that don't match the package manager
    if workspace.is_root && file_name.starts_with("package.json.") && file_name.ends_with(".tera") {
        let pm_suffix = file_name
            .trim_end_matches(".tera")
            .split('.')
            .last()
            .unwrap_or("");
        if pm_suffix != package_manager && pm_suffix != "base" {
            log_debug(&format!(
                "Skipping non-matching package.json template: {}",
                file_name
            ));
            return Ok(());
        }
    }
    log_debug(&format!("Copying non-template file: {}", file_name));

    let dest_path = workspace
        .dest_path
        .join(path.strip_prefix(&workspace.source_path)?);

    fs::create_dir_all(dest_path.parent().unwrap())?;
    fs::copy(path, &dest_path).map_err(|e| {
        anyhow::anyhow!(
            "Failed to copy file from {} to {}: {}",
            path.display(),
            dest_path.display(),
            e
        )
    })?;

    log_debug(&format!(
        "Copied file: {} -> {}",
        path.display(),
        dest_path.display()
    ));
    Ok(())
}

pub fn get_workspaces(template_dir: &Path, project_dir: &Path) -> Vec<Workspace> {
    // bare minimum workspaces
    vec![
        Workspace {
            name: "root".to_string(),
            source_path: template_dir.to_path_buf(),
            dest_path: project_dir.to_path_buf(),
            is_root: true,
        },
        Workspace {
            name: "web".to_string(),
            source_path: template_dir.join("apps/web"),
            dest_path: project_dir.join("apps/web"),
            is_root: false,
        },
        Workspace {
            name: "api".to_string(),
            source_path: template_dir.join("apps/api"),
            dest_path: project_dir.join("apps/api"),
            is_root: false,
        },
        Workspace {
            name: "app".to_string(),
            source_path: template_dir.join("apps/app"),
            dest_path: project_dir.join("apps/app"),
            is_root: false,
        },
        Workspace {
            name: "ui".to_string(),
            source_path: template_dir.join("packages/ui"),
            dest_path: project_dir.join("packages/ui"),
            is_root: false,
        },
        Workspace {
            name: "logger".to_string(),
            source_path: template_dir.join("packages/logger"),
            dest_path: project_dir.join("packages/logger"),
            is_root: false,
        },
        // Add more workspaces as needed
    ]
}

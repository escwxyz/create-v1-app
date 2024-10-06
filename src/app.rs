use anyhow::Result;
use console::style;
use indicatif::HumanDuration;
use std::{fs, path::Path, time::Instant};
use tera::Context;

use crate::cli::Service;
use crate::logger::log_debug;
use crate::workspace::{get_workspaces, Workspace};
use crate::{
    logger::log_info,
    tera::{initialize_tera, TERA},
    workspace::process_workspace,
};

pub fn create_new_app(
    name: &str,
    services: &[Service],
    package_manager: Option<&str>,
) -> Result<()> {
    let start_time = Instant::now();

    let package_manager = crate::utils::confirm_package_manager(package_manager)?;
    log_info(&format!("Using package manager: {}", package_manager));

    // init tera
    initialize_tera()?;

    let tera = TERA.lock().unwrap();

    log_debug(&format!(
        "Creating new app: {} with {} service(s) by {}",
        name,
        services.len(),
        package_manager
    ));

    let mut context = Context::new();
    context.insert("project_name", name);
    // context.insert(
    //     "services",
    //     &services
    //         .iter()
    //         .map(|s| s.to_string())
    //         .collect::<Vec<String>>(),
    // );
    context.insert("package_manager", &package_manager);

    let project_path = Path::new(name);
    fs::create_dir_all(project_path)?;

    let apps_path = project_path.join("apps");
    fs::create_dir_all(&apps_path)?;

    let packages_path = project_path.join("packages");
    fs::create_dir_all(&packages_path)?;

    let templates_root = Path::new("templates");

    let mut workspaces = get_workspaces(templates_root, project_path);

    // if we specify some services, we add them to the workspace
    if !services.is_empty() {
        for service in services {
            let workspace = Workspace {
                name: service.to_string(),
                source_path: templates_root.join("services").join(service.to_string()),
                dest_path: project_path.join("packages").join(service.to_string()),
                is_root: false,
            };
            workspaces.push(workspace);
        }
    }

    let total_steps = workspaces.len() + 1;

    for (step, workspace) in workspaces.iter().enumerate() {
        log_info(&format!(
            "[{}/{}] Processing workspace: {}",
            step + 1,
            total_steps,
            workspace.name
        ));
        process_workspace(
            &workspace,
            &tera,
            &context,
            &package_manager,
            templates_root,
        )?;
    }

    log_info(&format!(
        "[{}/{}] Installing dependencies...",
        total_steps, total_steps
    ));
    // Install dependencies
    // install_dependencies(project_path, &package_manager)?;

    println!(
        "{}{}",
        style("V1 app created successfully! in ").bold().dim(),
        HumanDuration(start_time.elapsed())
    );

    Ok(())
}

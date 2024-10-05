use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::Instant,
};

use anyhow::Result;
use console::style;
use dialoguer::{Input, MultiSelect};
use indicatif::HumanDuration;
use tera::Context;
use walkdir::WalkDir;

use crate::tera::{initialize_tera, TERA};

pub fn create_new_app(
    name: &str,
    services: &[String],
    package_manager: Option<&str>,
) -> Result<()> {
    let start_time = Instant::now();

    // TODO: indicator for creating new V1 app
    // TODO: divided into multiple steps

    // log_debug(&format!("Creating new V1 app: {}", name));
    // log_info(&format!("Services: {:?}", services));

    let package_manager = confirm_package_manager(package_manager)?;
    // log_info(&format!("Using package manager: {}", package_manager));

    // init tera
    initialize_tera()?;

    let mut context = Context::new();
    context.insert("project_name", name);
    context.insert("services", services);
    context.insert("package_manager", &package_manager);

    let project_path = Path::new(name);
    fs::create_dir_all(project_path)?;

    let apps_path = project_path.join("apps");
    fs::create_dir_all(&apps_path)?;

    render_tera_templates(project_path, &context)?;

    copy_non_template_files(&PathBuf::from("templates"), project_path)?;

    let packages_path = project_path.join("packages");
    fs::create_dir_all(&packages_path)?;

    // Add selected services
    for service in services {
        add_service(project_path, service, &context)?;
    }

    // Install dependencies
    // TODO: Install dependencies asynchronously
    // TODO: Add a progress bar
    // TODO: temep disactive
    // install_dependencies(project_path, &package_manager)?;

    println!(
        "{}{}",
        style("V1 app created successfully! in ").bold().dim(),
        HumanDuration(start_time.elapsed())
    );

    Ok(())
}

pub fn run_interactive_dialogue() -> Result<()> {
    let selection = dialoguer::Select::new()
        .with_prompt("What would you like to do?") // TODO style
        .items(&["Create a new V1 app", "Add a service to an existing V1 app"])
        .default(0)
        .interact()
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    match selection {
        0 => {
            // create new app
            let name: String = Input::new()
                .with_prompt("Enter the project name") // TODO style
                .interact()
                .map_err(|e| anyhow::anyhow!(e.to_string()))?;

            let services = select_services()?;
            let package_manager = select_package_manager()?;

            create_new_app(&name, &services, Some(&package_manager))
        }
        1 => {
            // add service to existing app
            let _service: String = Input::new()
                .with_prompt("Enter the service name") // TODO style
                .interact()
                .map_err(|e| anyhow::anyhow!(e.to_string()))?;
            // TODO: we need to get the project_name and project_path (current working directory)
            // add_service(&service)
            Ok(())
        }
        _ => unreachable!(),
    }
}

pub fn select_package_manager() -> Result<String> {
    let package_managers = vec!["npm", "yarn", "pnpm", "bun"];
    let selection = dialoguer::Select::new()
        .with_prompt("Select a package manager") // TODO style
        .items(&package_managers)
        .default(0)
        .interact()
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    Ok(package_managers[selection].to_string())
}

pub fn select_services() -> Result<Vec<String>> {
    // TODO: shall be constant or enum?
    let services = vec![
        "cal",
        "dub",
        "openpanel",
        "resend",
        "trigger",
        "sentry",
        "upstash",
    ];
    let selections = MultiSelect::new()
        .with_prompt("Select services to add") // TODO style
        .items(&services)
        .interact()
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    Ok(selections
        .into_iter()
        .map(|i| services[i].to_string())
        .collect())
}

fn confirm_package_manager(package_manager: Option<&str>) -> Result<String> {
    match package_manager {
        Some(pm) => Ok(pm.to_string()),
        None => select_package_manager(),
    }
}

fn render_tera_templates(project_path: &Path, context: &tera::Context) -> Result<()> {
    // TODO make it more decoupled
    let tera = TERA.lock().unwrap();
    let package_manager = context
        .get("package_manager")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    for template_name in tera.get_template_names().collect::<Vec<&str>>() {
        // Skip package.json.*tera files that don't match the selected package manager
        if template_name.starts_with("package.json.") && template_name != "package.json.tera" {
            let pm_suffix = template_name
                .trim_end_matches(".tera")
                .split('.')
                .last()
                .unwrap_or("");
            if pm_suffix != package_manager {
                continue;
            }
        }

        // Skip pnpm-workspace.yaml if package manager is not pnpm
        if template_name == "pnpm-workspace.yaml.tera" && package_manager != "pnpm" {
            continue;
        }

        let content = tera
            .render(template_name, context)
            .map_err(|e| anyhow::anyhow!("Failed to render template {}: {}", template_name, e))?;

        // Skip empty templates (conditionally excluded)
        if content.trim().is_empty() {
            continue;
        }

        let mut file_path = project_path.join(template_name.trim_end_matches(".tera"));

        // Rename package.json.{pm}.tera to package.json
        if template_name.starts_with("package.json.") && template_name != "package.json.tera" {
            file_path = project_path.join("package.json");
        }

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(file_path, content)
            .map_err(|e| anyhow::anyhow!("Failed to write file {}: {}", template_name, e))?;
    }
    Ok(())
}

#[allow(dead_code)]
fn install_dependencies(project_path: &Path, package_manager: &str) -> Result<()> {
    let install_command = match package_manager {
        "npm" => "install",
        "yarn" => "install",
        "pnpm" => "install",
        "bun" => "install",
        _ => return Err(anyhow::anyhow!("Unsupported package manager")),
    };

    // TODO progress & multiple threads?
    // https://github.com/console-rs/indicatif/blob/HEAD/examples/yarnish.rs

    Command::new(package_manager)
        .arg(install_command)
        .current_dir(project_path)
        .output()
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    Ok(())
}

pub fn add_service(project_path: &Path, service: &str, context: &Context) -> Result<()> {
    // log_info(&format!("Adding service: {}", service));

    let service_src = PathBuf::from("templates").join("services").join(service);
    let service_dst = project_path.join("packages").join(service);

    // Create the service directory
    fs::create_dir_all(&service_dst)?;

    // Copy non-template files
    copy_service_files(&service_src, &service_dst)?;

    // Render service-specific templates
    render_service_templates(&service_src, &service_dst, context)?; // TODO context: only project_name is needed

    // log_info(&format!("Service {} added successfully", service));
    Ok(())
}

fn copy_service_files(src_dir: &Path, dst_dir: &Path) -> std::io::Result<()> {
    for entry in WalkDir::new(src_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let is_tera = entry.path().extension().map_or(false, |ext| ext == "tera");

            if !is_tera {
                let relative_path = entry.path().strip_prefix(src_dir).unwrap();
                let dst_path = dst_dir.join(relative_path);
                if let Some(parent) = dst_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(entry.path(), dst_path)?;
            }
        }
    }
    Ok(())
}

fn render_service_templates(src_dir: &Path, dst_dir: &Path, context: &Context) -> Result<()> {
    let tera = TERA.lock().unwrap();

    for entry in WalkDir::new(src_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file()
            && entry.path().extension().map_or(false, |ext| ext == "tera")
        {
            let relative_path = entry.path().strip_prefix(src_dir).unwrap();
            let template_name = relative_path.to_str().unwrap();

            let content = tera.render(template_name, context).map_err(|e| {
                anyhow::anyhow!(format!(
                    "Failed to render template {}: {}",
                    template_name, e
                ))
            })?;

            let dst_path = dst_dir.join(relative_path.with_extension(""));
            if let Some(parent) = dst_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(dst_path, content).map_err(|e| {
                anyhow::anyhow!(format!("Failed to write file {}: {}", template_name, e))
            })?;
        }
    }
    Ok(())
}

fn copy_non_template_files(src_dir: &Path, dst_dir: &Path) -> std::io::Result<()> {
    // TODO: only copy files from services which are added
    for entry in WalkDir::new(src_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let is_tera = entry.path().extension().map_or(false, |ext| ext == "tera");

            if !is_tera {
                let relative_path = entry.path().strip_prefix(src_dir).unwrap();
                let dst_path = dst_dir.join(relative_path);
                if let Some(parent) = dst_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(entry.path(), dst_path)?;
            }
        }
    }
    Ok(())
}

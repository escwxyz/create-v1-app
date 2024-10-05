use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
    thread,
    time::Instant,
};

use anyhow::anyhow;
use anyhow::Result;
use console::style;
use dialoguer::{Input, MultiSelect};
use indicatif::{HumanDuration, MultiProgress, ProgressBar, ProgressStyle};
use log::{debug, error, info};
use tera::Context;
use walkdir::WalkDir;

use crate::{
    logger::{log_debug, log_error, log_info},
    tera::{initialize_tera, TERA},
};

pub fn create_new_app(
    name: &str,
    services: &[String],
    package_manager: Option<&str>,
) -> Result<()> {
    let start_time = Instant::now();
    let spinner_style = ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")
        .unwrap()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");

    // TODO: indicator for creating new V1 app
    // TODO: divided into multiple steps

    log_debug(&format!("Creating new V1 app: {}", name));
    log_info(&format!("Services: {:?}", services));

    let package_manager = confirm_package_manager(package_manager)?;
    log_info(&format!("Using package manager: {}", package_manager));

    // init tera
    initialize_tera()?;

    let mut context = Context::new();
    context.insert("project_name", name);
    context.insert("services", services);
    context.insert("package_manager", &package_manager);

    // let total_steps = 4 + services.len();

    // log_info(&format!(
    //     "[1/{}] Creating project structure...",
    //     total_steps
    // ));

    let project_path = Path::new(name);
    fs::create_dir_all(project_path)?;

    let apps_path = project_path.join("apps");
    fs::create_dir_all(&apps_path)?;

    render_tera_templates(project_path, &context)?;

    copy_non_template_files(&PathBuf::from("templates"), project_path)?;

    let packages_path = project_path.join("packages");
    fs::create_dir_all(&packages_path)?;

    // Add selected services
    for (step, service) in services.iter().enumerate() {
        // log_info(&format!(
        //     "[{}/{}] Adding service: {}",
        //     step + 2,
        //     total_steps,
        //     service
        // ));
        add_service(project_path, service, &context)?;
    }

    // log_info(&format!("[3/{}] Installing dependencies...", total_steps));
    // Install dependencies
    // TODO: Install dependencies asynchronously
    // TODO: Add a progress bar
    // TODO: temep disactive
    install_dependencies(project_path, &package_manager)?;

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

pub fn install_dependencies(project_path: &Path, package_manager: &str) -> Result<()> {
    let install_command = match package_manager {
        "npm" => "install",
        "yarn" => "install",
        "pnpm" => "install",
        "bun" => "install",
        _ => return Err(anyhow!("Unsupported package manager")),
    };

    let multi_progress = Arc::new(MultiProgress::new());
    let style = ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
        .unwrap()
        .progress_chars("##-");

    let workspaces = get_workspaces(project_path)?;
    let mut handles = vec![];

    for workspace in workspaces {
        let mp = multi_progress.clone();
        let pb = mp.add(ProgressBar::new(100));
        pb.set_style(style.clone());
        pb.set_message(format!("Installing {}", workspace.display()));

        let package_manager = package_manager.to_string();
        let install_command = install_command.to_string();
        let workspace_path = workspace.to_path_buf();

        let handle = thread::spawn(move || {
            log_debug(&format!(
                "Starting installation for workspace: {}",
                workspace_path.display()
            ));
            log_debug(&format!(
                "Running command: {} {}",
                package_manager, install_command
            ));

            let output = Command::new(&package_manager)
                .arg(&install_command)
                .current_dir(&workspace_path)
                .output()?;

            log_debug(&format!(
                "Installation command completed for {}",
                workspace_path.display()
            ));

            if !output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                log_error(&format!(
                    "Installation failed in {}:\nStdout: {}\nStderr: {}",
                    workspace_path.display(),
                    stdout,
                    stderr
                ));
                return Err(anyhow!(
                    "Installation failed in {}:\nStdout: {}\nStderr: {}",
                    workspace_path.display(),
                    stdout,
                    stderr
                ));
            }

            let node_modules_path = workspace_path.join("node_modules");
            if !node_modules_path.exists() {
                return Err(anyhow!(
                    "node_modules not created in {}",
                    workspace_path.display()
                ));
            }

            pb.finish_with_message(format!("Installed {}", workspace_path.display()));
            Ok(())
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

    Ok(())
}

pub fn add_service(project_path: &Path, service: &str, context: &Context) -> Result<()> {
    log_info(&format!("Adding service: {}", service));

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

fn get_workspaces(project_path: &Path) -> Result<Vec<PathBuf>> {
    Ok(vec![
        project_path.to_path_buf(), // Include the root project
        project_path.join("apps/web"),
        project_path.join("apps/api"),
        project_path.join("apps/app"),
        // TODO need a better way to get all packages, we get services from args
    ])
}

use std::path::Path;

use anyhow::Result;
use clap::ValueEnum;
use dialoguer::{theme::ColorfulTheme, Confirm, MultiSelect};
use tera::Context;

use crate::{
    cli::Service,
    logger::log_info,
    tera::TERA,
    utils::{get_package_json, PackageJson},
    workspace::{process_workspace, Workspace},
};

pub fn select_services() -> Result<Vec<Service>> {
    let add_services = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Do you want to add any services?")
        .default(true)
        .interact()
        .map_err(|e| anyhow::anyhow!("Failed to prompt for service addition: {}", e))?;

    if !add_services {
        return Ok(Vec::new());
    }

    let services: Vec<String> = Service::value_variants()
        .into_iter()
        .map(|v| {
            let name = v.to_string();
            let help = v
                .to_possible_value()
                .unwrap()
                .get_help()
                .unwrap()
                .to_string();
            format!("{name}: {help}")
        })
        .collect();

    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select services to add")
        .items(&services)
        .interact()
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    let result = selections
        .into_iter()
        .map(|i| services[i].to_string())
        .map(|s| {
            let name = s.split(':').next().unwrap();
            Service::from_str(name, false).expect("Invalid service")
        })
        .collect::<Vec<_>>();

    Ok(result)
}

pub fn add_services(services: &[Service], templates_root: &Path) -> Result<()> {
    let tera = TERA.lock().unwrap();

    let PackageJson {
        name,
        package_manager,
    } = get_package_json(None)?;

    let mut context = Context::new();
    context.insert("project_name", &name);
    context.insert("package_manager", &package_manager);

    let workspaces = services
        .iter()
        .map(|service| Workspace {
            name: service.to_string(),
            source_path: templates_root.join("services").join(service.to_string()),
            dest_path: Path::new(&name).join("packages").join(service.to_string()),
            is_root: false,
        })
        .collect::<Vec<Workspace>>();

    for workspace in workspaces {
        log_info(&format!("Adding service: {}", workspace.name));
        process_workspace(
            &workspace,
            &tera,
            &context,
            &package_manager,
            templates_root,
        )?;
    }

    Ok(())
}

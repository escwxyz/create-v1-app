use anyhow::Result;

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

pub fn confirm_package_manager(package_manager: Option<&str>) -> Result<String> {
    match package_manager {
        Some(pm) => Ok(pm.to_string()),
        None => select_package_manager(),
    }
}

// pub fn install_dependencies(project_path: &Path, package_manager: &str) -> Result<()> {
//     let install_command = match package_manager {
//         "npm" => "install",
//         "yarn" => "install",
//         "pnpm" => "install",
//         "bun" => "install",
//         _ => return Err(anyhow!("Unsupported package manager")),
//     };

//     let multi_progress = Arc::new(MultiProgress::new());
//     let style = ProgressStyle::default_bar()
//         .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
//         .unwrap()
//         .progress_chars("##-");

//     let workspaces = get_workspaces(project_path)?;
//     let mut handles = vec![];

//     for workspace in workspaces {
//         let mp = multi_progress.clone();
//         let pb = mp.add(ProgressBar::new(100));
//         pb.set_style(style.clone());
//         pb.set_message(format!("Installing {}", workspace.display()));

//         let package_manager = package_manager.to_string();
//         let install_command = install_command.to_string();
//         let workspace_path = workspace.to_path_buf();

//         let handle = thread::spawn(move || {
//             log_debug(&format!(
//                 "Starting installation for workspace: {}",
//                 workspace_path.display()
//             ));
//             log_debug(&format!(
//                 "Running command: {} {}",
//                 package_manager, install_command
//             ));

//             let output = Command::new(&package_manager)
//                 .arg(&install_command)
//                 .current_dir(&workspace_path)
//                 .output()?;

//             log_debug(&format!(
//                 "Installation command completed for {}",
//                 workspace_path.display()
//             ));

//             if !output.status.success() {
//                 let stdout = String::from_utf8_lossy(&output.stdout);
//                 let stderr = String::from_utf8_lossy(&output.stderr);
//                 log_error(&format!(
//                     "Installation failed in {}:\nStdout: {}\nStderr: {}",
//                     workspace_path.display(),
//                     stdout,
//                     stderr
//                 ));
//                 return Err(anyhow!(
//                     "Installation failed in {}:\nStdout: {}\nStderr: {}",
//                     workspace_path.display(),
//                     stdout,
//                     stderr
//                 ));
//             }

//             let node_modules_path = workspace_path.join("node_modules");
//             if !node_modules_path.exists() {
//                 return Err(anyhow!(
//                     "node_modules not created in {}",
//                     workspace_path.display()
//                 ));
//             }

//             pb.finish_with_message(format!("Installed {}", workspace_path.display()));
//             Ok(())
//         });

//         handles.push(handle);
//     }

//     let results: Vec<Result<()>> = handles
//         .into_iter()
//         .map(|h| h.join().unwrap_or(Err(anyhow!("Thread panicked"))))
//         .collect();

//     for result in &results {
//         if let Err(e) = result {
//             eprintln!("Installation error: {}", e);
//         }
//     }

//     if results.iter().any(|r| r.is_err()) {
//         return Err(anyhow!("One or more installations failed"));
//     }

//     Ok(())
// }

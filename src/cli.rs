use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::utils::{create_new_app, run_interactive_dialogue};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(name = "create-v1-app")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Create a new V1 app")]
    New {
        #[arg(help = "The name of the new project")]
        name: String,

        #[arg(long, help = "A list of services to add to the project")]
        services: Option<Vec<String>>,

        #[arg(long, help = "The package manager to use for the project")]
        package_manager: Option<String>,
    },
    #[command(about = "Add a service or provider to an existing V1 app")]
    Add {
        #[command(subcommand)]
        subcommand: AddSubcommands,
    },
}

#[derive(Subcommand)]
enum AddSubcommands {
    #[command(about = "Add a service to an existing V1 app")]
    Service {
        #[arg(help = "The name of the service to add")]
        service_name: String,
    },
    #[command(about = "Add a provider to an existing V1 app")]
    Provider {
        #[arg(help = "The name of the provider to add")]
        name: String,
    },
}

pub fn parse_cli(args: Vec<String>) -> Result<()> {
    if args.is_empty() {
        return run_interactive_dialogue();
    }

    let cli = Cli::parse_from(args);

    match cli.command {
        Some(Commands::New {
            name,
            services,
            package_manager,
        }) => {
            let services = services.unwrap_or_default();

            let package_manager = package_manager.unwrap_or("npm".to_string());

            create_new_app(&name, &services, Some(&package_manager))
        }
        Some(Commands::Add { subcommand }) => {
            // TODO: we should also know which package manager the user uses
            match subcommand {
                AddSubcommands::Service { service_name: _ } => {
                    // TODO: we need to get the project_name and project_path (current working directory)
                    // add_service(&service_name)
                    Ok(())
                }
                AddSubcommands::Provider { name: _ } => {
                    // TODO: Implement provider addition logic
                    Ok(())
                }
            }
        }
        None => run_interactive_dialogue(),
    }
}

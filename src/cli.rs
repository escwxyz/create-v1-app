use std::path::Path;
use std::str::FromStr;

use anyhow::Result;
use clap::{Args, Parser, Subcommand, ValueEnum};
use dialoguer::theme::ColorfulTheme;
use dialoguer::Input;

use crate::app::create_new_app;
use crate::logger::log_debug;
use crate::service::add_services;
use crate::{service::select_services, utils::select_package_manager};

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

        #[arg(short, long, help = "A list of services to add to the project")]
        services: Option<Vec<Service>>,

        #[arg(short, long, help = "The package manager to use for the project")]
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
    Services(ServicesArgs),
    #[command(about = "Add a provider to an existing V1 app")]
    Provider {
        #[arg(help = "The name of the provider to add")]
        name: String,
    },
}

#[derive(Args)]
pub struct ServicesArgs {
    #[arg(help = "The names of the services to add", value_delimiter = ',')]
    services: Vec<Service>,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Service {
    #[clap(help = "Calendar service for scheduling")]
    Cal,
    #[clap(help = "URL shortener and link management")]
    Dub,
    #[clap(help = "Open-source control panel")]
    Openpanel,
    #[clap(help = "Email API that enables email sending")]
    Resend,
    #[clap(help = "Workflow automation platform")]
    Trigger,
    #[clap(help = "Application monitoring and error tracking")]
    Sentry,
    #[clap(help = "Serverless database for Redis and Kafka")]
    Upstash,
}

impl FromStr for Service {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "cal" => Ok(Service::Cal),
            "dub" => Ok(Service::Dub),
            "openpanel" => Ok(Service::Openpanel),
            "resend" => Ok(Service::Resend),
            "trigger" => Ok(Service::Trigger),
            "sentry" => Ok(Service::Sentry),
            "upstash" => Ok(Service::Upstash),
            _ => Err(format!("Unknown service: {}", s)),
        }
    }
}

impl ToString for Service {
    fn to_string(&self) -> String {
        match self {
            Service::Cal => "cal",
            Service::Dub => "dub",
            Service::Openpanel => "openpanel",
            Service::Resend => "resend",
            Service::Trigger => "trigger",
            Service::Sentry => "sentry",
            Service::Upstash => "upstash",
        }
        .to_string()
    }
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
        Some(Commands::Add { subcommand }) => match subcommand {
            AddSubcommands::Services(services) => {
                log_debug(&format!("Adding services: {}", services.services.len()));
                let templates_root = Path::new("templates");
                add_services(&services.services, &templates_root)?;
                Ok(())
            }
            _ => unreachable!(),
        },
        None => run_interactive_dialogue(),
    }
}

fn run_interactive_dialogue() -> Result<()> {
    let selection = dialoguer::Select::with_theme(&ColorfulTheme::default())
        .with_prompt("What would you like to do?")
        .items(&["Create a new v1 app", "Add a service to an existing v1 app"])
        .default(0)
        .interact()
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    match selection {
        0 => {
            // create new app
            let name: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter the project name")
                .interact()
                .map_err(|e| anyhow::anyhow!(e.to_string()))?;

            let services = select_services()?;
            let package_manager = select_package_manager()?;

            create_new_app(&name, &services, Some(&package_manager))
        }
        1 => {
            // add services to existing app
            let services = select_services()?;
            let templates_root = Path::new("templates");
            add_services(&services, &templates_root)?;
            Ok(())
        }
        _ => unreachable!(),
    }
}

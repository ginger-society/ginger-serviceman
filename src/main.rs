use core::fmt;
use std::path::Path;

use clap::{Parser, Subcommand, ValueEnum};
use generate::generate_arbitrary_client;
use init::initialize;
use service::generate_client;
use utils::{fetch_metadata_and_process, LANG};
use IAMService::apis::configuration::Configuration as IAMConfiguration;
use IAMService::{
    apis::{configuration::Configuration, default_api::identity_validate_token},
    get_configuration as get_iam_configuration,
};
use MetadataService::{
    apis::configuration::Configuration as MetadataConfiguration,
    get_configuration as get_metadata_configuration,
};

mod generate;
mod init;
mod service;
mod utils;

/// Command line interface for managing the application
#[derive(Parser)]
#[clap(name = "CLI")]
#[clap(about = "A CLI for managing service dependencies", long_about = None)]
struct CLI {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Fetch metadata and process it
    Init {
        #[clap(value_parser)]
        repo_path: String,
    },
    /// Configures a service to a project
    Config,
    /// Connect to an environment
    Connect {
        #[clap(value_enum, default_value_t=Environment::Dev)]
        env: Environment,
    },
    /// Generate a client for a specified language
    Generate {
        #[clap(value_enum)]
        lang: LANG,
        #[clap(value_parser)]
        swagger_path: String,
        #[clap(value_parser)]
        server_url: String,
        #[clap(value_parser)]
        out_folder: String,
    },
}

#[derive(ValueEnum, Clone)]
enum Environment {
    Dev,
    Stage,
    Prod,
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Environment::Dev => write!(f, "dev"),
            Environment::Stage => write!(f, "stage"),
            Environment::Prod => write!(f, "prod"),
        }
    }
}

#[tokio::main]
async fn check_session_gurad(
    cli: CLI,
    config_path: &Path,
    iam_config: &IAMConfiguration,
    metadata_config: &MetadataConfiguration,
) {
    match identity_validate_token(&iam_config).await {
        Ok(response) => {
            match &cli.command {
                Commands::Config {} => {
                    fetch_metadata_and_process(config_path, &iam_config, &metadata_config).await;
                }
                Commands::Connect { env } => generate_client(config_path, env.clone()),
                Commands::Init { repo_path } => initialize(repo_path, config_path),
                Commands::Generate {
                    lang,
                    swagger_path,
                    server_url,
                    out_folder,
                } => {
                    generate_arbitrary_client(swagger_path, lang.clone(), server_url, out_folder);
                }
            };

            println!("Token is valid: {:?}", response)
        }
        Err(error) => {
            println!("Token validation failed: {:?}", error);
            std::process::exit(1);
        }
    }
}

fn main() {
    let cli = CLI::parse();

    let config_path = Path::new("services.toml");

    let iam_config: IAMConfiguration = get_iam_configuration();
    let metadata_config: MetadataConfiguration = get_metadata_configuration();
    check_session_gurad(cli, config_path, &iam_config, &metadata_config);
}

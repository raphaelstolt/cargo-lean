pub mod create;
pub mod lint;
pub mod update;

pub(crate) mod gitignore;
pub(crate) mod utils;

use anyhow::Result;
use clap::Subcommand;

use self::create::CreateArgs;
use self::lint::LintArgs;
use self::update::UpdateArgs;

#[derive(Subcommand)]
pub enum Commands {
    /// Check if all unnecessary repository artefacts are listed in Cargo.toml exclude
    Lint(LintArgs),
    /// Add unnecessary repository artefacts to Cargo.toml exclude list
    Create(CreateArgs),
    /// Update Cargo.toml exclude list with current repository artefacts
    Update(UpdateArgs),
}

impl Commands {
    pub fn run(&self) -> Result<()> {
        match self {
            Commands::Lint(args) => args.run(),
            Commands::Create(args) => args.run(),
            Commands::Update(args) => args.run(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "text" => Ok(OutputFormat::Text),
            "json" => Ok(OutputFormat::Json),
            _ => Err(format!(
                "Invalid format: {}. Supported formats: text, json",
                s
            )),
        }
    }
}

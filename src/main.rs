use clap::{Parser, Subcommand};
use minijinja::Environment;
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};
use uuid::Uuid;

#[derive(Parser)]
#[command(about = "The system management tool for Maple Linux", version)]
struct Arguments {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Reconfigures the system using /etc/maple.toml")]
    Reconfigure,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
struct Configuration {
    hostname: String,
    partition: Vec<PartitionConfiguration>
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            hostname: String::from("maple"),
            partition: Vec::new()
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct PartitionConfiguration {
    format: String,
    #[serde(default = "partition_default_fsck")]
    fsck: usize,
    id: Uuid,
    mountpoint: Option<String>,
    #[serde(default = "partition_default_options")]
    options: Vec<String>
}

const fn partition_default_fsck() -> usize { 2 }
fn partition_default_options() -> Vec<String> { vec![] }

fn main() -> Result<(), Box<dyn Error>> {
    let args = Arguments::parse();
    match args.command {
        Commands::Reconfigure => reconfigure(&args)?,
    }
    Ok(())
}

fn reconfigure(_args: &Arguments) -> Result<(), Box<dyn Error>> {
    let config = toml::from_str::<Configuration>(&fs::read_to_string("/etc/maple.toml")?)?;
    let maple_d = Path::new("/etc/maple.d");
    reconfigure_tree(&config, maple_d, maple_d)?;
    Ok(())
}

fn reconfigure_target(maple_d: &Path, template: &Path) -> Result<PathBuf, Box<dyn Error>> {
    Ok(Path::new("/").join(template.strip_prefix(maple_d)?))
}

fn reconfigure_tree(config: &Configuration, maple_d: &Path, directory: &Path) -> Result<(), Box<dyn Error>> {
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_dir() {
            reconfigure_tree(config, maple_d, &path)?;
        } else {
            let mut engine = Environment::new();
            let target = reconfigure_target(maple_d, &path)?;
            let template_raw = fs::read_to_string(&path)?;
            engine.add_template("template", &template_raw)?;
            let template = engine.get_template("template")?;
            let rendered = template.render(config)?;
            fs::write(target, rendered)?;
        }
    }
    Ok(())
}

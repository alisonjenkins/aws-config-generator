use std::fs::File;
use std::io::prelude::*;
use std::process;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ConfigGeneratorConfig {
    pub master_account: MasterAccount,
    pub sso_options: SSOOptions,
    pub aws_cli_options: AWSCLIOptions,
}

#[derive(Debug, Deserialize)]
pub struct SSOOptions {
    pub sso_url: String,
    pub sso_region: String,
    pub sso_role: String,
}

#[derive(Debug, Deserialize)]
pub struct MasterAccount {
    pub account_id: String,
    pub account_name: String,
}

#[derive(Debug, Deserialize)]
pub struct AWSCLIOptions {
    pub default_region: String,
    pub default_output_type: String,
}


fn find_config() -> Result<std::path::PathBuf, &'static str> {
    let mut config_paths = vec![std::path::PathBuf::from("config.toml")];
    let config_path: std::path::PathBuf;
    if let Some(confdir) = dirs::config_dir() {
        let pos_config_path = confdir.join("aws-config-generator/config.toml");
        config_paths.push(pos_config_path);
    }
    // *TODO* Implement config file finding code!
    for check_config_path in config_paths {
        if check_config_path.exists() {
            config_path = check_config_path;
            return Ok(config_path);
        }
    }

    Err("Config file not found")
}

fn read_config(config_path: &std::path::PathBuf) -> Result<String, std::io::Error> {
    let mut file = File::open(config_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

fn parse_config(config_string: &str) -> Result<Box<ConfigGeneratorConfig>, String> {
    let config: Box<ConfigGeneratorConfig> = match toml::from_str(config_string) {
        Ok(parsed) => Box::new(parsed),
        Err(err) => return Err(format!("{}", err)),
    };

    Ok(config)
}

pub fn get_config() -> Box<ConfigGeneratorConfig> {
    let config_path = match find_config() {
        Ok(config_path) => config_path,
        Err(err) => {
            eprintln!("Unable to find config file: {}", err);
            process::exit(1);
        }
    };

    let config_content = match read_config(&config_path) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Unable to read the config file: {}", err);
            process::exit(1);
        }
    };

    match parse_config(&config_content) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Unable to parse the config file: {}", err);
            process::exit(1);
        }
    }
}

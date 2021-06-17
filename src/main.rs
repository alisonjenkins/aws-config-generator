use std::borrow::Borrow;
use std::default::Default;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::process;

use rusoto_core::Region;
use rusoto_organizations::{Account, ListAccountsRequest, Organizations, OrganizationsClient};
use rusoto_sts::{GetCallerIdentityRequest, Sts, StsClient};

async fn generate_aws_config(
    org_main_account: &String,
    default_region: &str,
    default_output_type: &str,
    sso_start_url: &str,
    sso_region: &str,
    sso_role_name: &str,
    accounts_list: &Vec<Account>,
) -> io::Result<String> {
    let mut config_string: String = format!(
        "[default]\nregion={}\noutput={}\n\n[profile main]\nsso_start_url = {}\nsso_region = {}\nregion = {}\noutput = {}\nsso_account_id = {}\nsso_role_name = {}\n\n",
        &default_region,
        &default_output_type,
        sso_start_url,
        sso_region,
        default_region,
        default_output_type,
        org_main_account,
        sso_role_name
    );

    for account in accounts_list {
        let mut account_name: String;
        let account_id: &String;

        match &account.name {
            Some(name) => {
                account_name = name.clone();
                account_name = account_name.replace(" ", "-").to_lowercase();
            }
            None => {
                eprintln!("No account Name!");
                std::process::exit(1);
            }
        }

        match &account.id {
            Some(id) => account_id = &id.borrow(),
            None => {
                eprintln!("No account ID!");
                std::process::exit(1);
            }
        }

        config_string = config_string + &format!(
            "[profile {}]\nsso_start_url = {}\nsso_region = {}\nregion = {}\noutput = {}\nsso_account_id = {}\nsso_role_name = {}\n\n",
            &account_name,
            &sso_start_url,
            &sso_region,
            &default_region,
            &default_output_type,
            &account_id,
            &sso_role_name
        );
    }

    Ok(config_string)
}

fn find_config() -> Result<std::path::PathBuf, &'static str> {
    let mut config_paths = vec![std::path::PathBuf::from("config.toml")];
    let config_path: std::path::PathBuf;
    match dirs::config_dir() {
        Some(confdir) => {
            let pos_config_path = confdir.join("aws-config-generator/config.toml");
            config_paths.push(pos_config_path);
        }
        _ => {}
    }
    // *TODO* Implement config file finding code!
    for check_config_path in config_paths {
        if check_config_path.exists() {
            config_path = std::path::PathBuf::from(check_config_path);
            return Ok(config_path);
        }
    }

    return Err("Config file not found");
}

fn read_config(config_path: &std::path::PathBuf) -> Result<String, std::io::Error> {
    let mut file = File::open(config_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

fn parse_config(config_string: &String) -> Result<Box<toml::Value>, Box<String>> {
    let config = match config_string.parse::<toml::Value>() {
        Ok(parsed) => Box::new(parsed),
        Err(err) => return Err(Box::new(format!("{}", err))),
    };

    Ok(config)
}

fn get_config() -> Box<toml::Value> {
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

    let config = match parse_config(&config_content) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Unable to parse the config file: {}", err);
            process::exit(1);
        }
    };

    config
}

#[tokio::main]
async fn main() -> () {
    let config = get_config();

    println!("{:?}", config);

    let org_client = OrganizationsClient::new(Region::default());
    let list_accounts_input: ListAccountsRequest = Default::default();

    let sts_client = StsClient::new(Region::default());
    let get_caller_identity_input = GetCallerIdentityRequest {};
    let org_main_account = match Sts::get_caller_identity(&sts_client, get_caller_identity_input)
        .await
    {
        Ok(resp) => resp,
        Err(err) => {
            eprintln!("STS Get Caller Identity failed: {}\nUnable to identify the organisation's main account.", err);
            std::process::exit(1);
        }
    };

    match org_client.list_accounts(list_accounts_input).await {
        Ok(output) => match output.accounts {
            Some(accounts_list) => {
                let config_string = generate_aws_config(
                    &org_main_account.account.unwrap(),
                    config["aws_cli_options"]["default_region"]
                        .as_str()
                        .unwrap(),
                    config["aws_cli_options"]["default_output_type"]
                        .as_str()
                        .unwrap(),
                    config["sso_options"]["sso_url"].as_str().unwrap(),
                    config["sso_options"]["sso_region"].as_str().unwrap(),
                    config["sso_options"]["sso_role"].as_str().unwrap(),
                    &accounts_list,
                )
                .await;
                println!("{}", config_string.unwrap());
            }

            None => println!("No accounts in organization"),
        },
        Err(error) => {
            println!("Error: {:?}", error);
        }
    }
    ()
}

use anyhow::Result;
use aws_config::retry::RetryConfig;

use aws_config_generator::configgen;
use std::collections::BTreeMap;

#[tokio::main]
async fn main() -> Result<()> {
    let _args = configgen::arg_parsing::get_args().await;
    let config = configgen::config::get_config();

    let shared_config = aws_config::load_from_env().await;
    let org_config = aws_sdk_organizations::config::Builder::from(&shared_config)
        .retry_config(RetryConfig::disabled())
        .build();
    let sts_config = aws_sdk_sts::config::Builder::from(&shared_config)
        .retry_config(RetryConfig::disabled())
        .build();
    let org_client = aws_sdk_organizations::Client::from_conf(org_config);
    let sts_client = aws_sdk_sts::Client::from_conf(sts_config);

    let org_main_account = match sts_client.get_caller_identity().send().await {
        Ok(resp) => resp,
        Err(err) => {
            eprintln!("Error: STS Get Caller Identity failed: {}\nUnable to identify the organisation's main account.", err);
            std::process::exit(1);
        }
    };

    let mut accounts_list = BTreeMap::new();
    let mut next_token: Option<String> = None;
    loop {
        match org_client
            .list_accounts()
            .set_next_token(next_token)
            .send()
            .await
        {
            Ok(output) => {
                next_token = output.next_token;

                match output.accounts {
                    Some(mut resp_accounts) => {
                        for account in resp_accounts.iter_mut() {
                            accounts_list.insert(
                                account
                                    .name
                                    .clone()
                                    .expect("Account missing name, accounts need to be named"),
                                account.clone(),
                            );
                        }
                    }
                    None => {}
                }

                if next_token == None {
                    break;
                }
            }
            Err(error) => {
                eprintln!("Error listing accounts: {:?}", error);
                std::process::exit(1);
            }
        }
    }

    let aws_cli_options = config.get("aws_cli_options").expect("aws_cli_options configuration section not found. Please see the example config and the README.md for instructions on how to configure this tool.");
    let sso_options = config.get("sso_options").expect("sso_options configuration section not found. Please see the example config and the README.md for instructions on how to configure this tool.");
    let name_by_account_name_tags: bool = match config.get("config") {
        Some(config_settings) => match config_settings.get("name_by_account_name_tags") {
            Some(should) => match should.as_bool() {
                Some(should) => should,
                None => {
                    eprintln!("Error: name_by_account_name_tags must be a boolean value.");
                    std::process::exit(1);
                }
            },
            None => false,
        },
        None => false,
    };

    let profile_template: String = match config.get("config") {
        Some(config_settings) => match config_settings.get("profile_template") {
            Some(template) => template
                .as_str()
                .expect("Unable to read the profile_template config option as a string")
                .to_string(),
            None => "basic_profile.txt".to_string(),
        },
        None => "basic_profile.txt".to_string(),
    };

    let config_string = configgen::generate::generate_aws_config(
        &org_main_account.account.unwrap(),
        aws_cli_options
            .get("default_region")
            .expect("aws_cli_options.default_region configuration file entry is missing")
            .as_str()
            .expect("failed to convert aws_cli_options.default_region to a &str"),
        aws_cli_options
            .get("default_output_type")
            .expect("aws_cli_options.default_output_type configuration option is missing")
            .as_str()
            .expect("failed to convert aws_cli_options.default_output_type to a &str"),
        sso_options
            .get("sso_url")
            .expect("sso_options.sso_url configuration file entry is missing.")
            .as_str()
            .expect("failed to convert sso_options.sso_url configuration file entry to a &str"),
        sso_options
            .get("sso_region")
            .expect("sso_options.sso_region configuration file entry is missing")
            .as_str()
            .expect("failed to convert sso_options.sso_region to a &str"),
        sso_options
            .get("sso_role")
            .expect("sso_options.sso_role configuration file entry is missing")
            .as_str()
            .expect("failed to convert sso_options.sso_role to a &str"),
        &accounts_list,
        &config,
        name_by_account_name_tags,
        org_client,
        profile_template,
    )
    .await;
    println!(
        "{}",
        config_string.expect("Failed to generate config string")
    );
    Ok(())
}

mod configgen;

use color_eyre::{Result, eyre::eyre};
use aws_sdk_organizations::types::Account;
use tokio_stream::StreamExt;
use crate::configgen::generate::generate_aws_config;

async fn get_main_account_id(sts_client: &aws_sdk_sts::Client) -> Result<String> {
    let caller_identity_output = sts_client
        .get_caller_identity()
        .send()
        .await?;

    caller_identity_output.account.ok_or_else(|| eyre!("Could not get ID for main AWS account"))
}

async fn get_accounts(org_client: &aws_sdk_organizations::Client) -> Result<Vec<Account>> {
    let mut all_accounts: Vec<Account> = Vec::new();
    let mut accounts_paginator = org_client.list_accounts().into_paginator().send();

    while let Some(accounts_page) = accounts_paginator.next().await {
        if let Some(accounts) = accounts_page?.accounts() {
            all_accounts.append(&mut accounts.into());
        }
    }

    Ok(all_accounts)
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let _args = configgen::arg_parsing::get_args().await;
    let config = configgen::config::get_config();
    let awsconfig = aws_config::load_from_env().await;

    let org_client = aws_sdk_organizations::Client::new(&awsconfig);
    let sts_client = aws_sdk_sts::Client::new(&awsconfig);

    let org_main_account = get_main_account_id(&sts_client).await?;
    let accounts = get_accounts(&org_client).await?;

    let generate_aws_config_input = configgen::generate::GenerateAWSConfigInput {
        org_main_account,
        default_region: config.aws_cli_options.default_region,
        default_output_type: config.aws_cli_options.default_output_type,
        sso_start_url: config.sso_options.sso_url,
        sso_region: config.sso_options.sso_region,
        sso_role_name: config.sso_options.sso_role,
        accounts_list: accounts
    };

    let config = generate_aws_config(generate_aws_config_input).await?;
    println!("{}", config);

    // let accounts = accounts_paginator.collect::<Result<Vec<Account>>>().await?;
    // let paginator = client.list_tables().into_paginator().items().send();
    // let table_names = paginator.collect::<Result<Vec<_>, _>>().await?;

    // let config_string = configgen::generate::generate_aws_config(
    //     &org_main_account,
    //     config["aws_cli_options"]["default_region"]
    //         .as_str()
    //         .unwrap(),
    //     config["aws_cli_options"]["default_output_type"]
    //         .as_str()
    //         .unwrap(),
    //     config["sso_options"]["sso_url"].as_str().unwrap(),
    //     config["sso_options"]["sso_region"].as_str().unwrap(),
    //     config["sso_options"]["sso_role"].as_str().unwrap(),
    //     &accounts,
    // )
    // .await;
    // println!("{}", config_string.unwrap());
    Ok(())
}

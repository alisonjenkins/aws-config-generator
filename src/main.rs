use std::default::Default;

use rusoto_core::Region;
use rusoto_organizations::{ListAccountsRequest, Organizations, OrganizationsClient};
use rusoto_sts::{GetCallerIdentityRequest, Sts, StsClient};

mod configgen;

#[tokio::main]
async fn main() -> () {
    let config = configgen::config::get_config();

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
                let config_string = configgen::generate::generate_aws_config(
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

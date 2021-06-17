use std::default::Default;

use rusoto_core::Region;
use rusoto_organizations::{Account, ListAccountsRequest, Organizations, OrganizationsClient};
use rusoto_sts::{GetCallerIdentityRequest, Sts, StsClient};

mod configgen;

#[tokio::main]
async fn main() -> () {
    let config = configgen::config::get_config();

    println!("{:?}", config);

    let org_client = OrganizationsClient::new(Region::default());

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

    let mut accounts_list = Vec::<Account>::new();
    let mut next_token: Option<String> = Some("".to_string());
    loop {
        let mut list_accounts_input: ListAccountsRequest = Default::default();
        println!("Next token is: '{}'\n", next_token.clone().unwrap());
        if next_token.clone().unwrap() != String::from("") {
            list_accounts_input.next_token = next_token.clone();
        }

        match org_client.list_accounts(list_accounts_input).await {
            Ok(output) => {
                next_token = match output.next_token {
                    Some(token) => Some(token),
                    None => Some(String::from("")),
                };

                match output.accounts {
                    Some(mut resp_accounts) => accounts_list.append(&mut resp_accounts),
                    None => {}
                }

                if next_token.clone().unwrap() == String::from("") {
                    println!("No more organisation account pages.");
                    break;
                } else {
                    println!("There are more organisation account pages... getting the next");
                }
                //             match output.accounts {
                //                 Some(resp_accounts_list) => {
                //
                //                 }
                //
                //                 None => {
                //                     println!("No accounts in organization"),
                //                 }
                //             }
            }
            Err(error) => {
                println!("Error: {:?}", error);
            }
        }
    }

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
    ()
}

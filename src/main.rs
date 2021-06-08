use std::borrow::Borrow;
use std::default::Default;
use std::io;

use rusoto_core::Region;
use rusoto_organizations::{Account, Organizations, OrganizationsClient, ListAccountsRequest};

async fn generate_aws_config(region: &str, output_type: &str, sso_start_url: &str, sso_region: &str, sso_role_name: &str, accounts_list: &Vec<Account>) -> io::Result<String> {
    let mut config_string: String = format!("[default] \nregion={}\noutput={}\n\n", &region, &output_type);

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
            &region,
            &output_type,
            &account_id,
            &sso_role_name
        );
    }


    Ok(config_string)
}

#[tokio::main]
async fn main() -> () {
    let org_client = OrganizationsClient::new(Region::default());
    let list_accounts_input: ListAccountsRequest = Default::default();

    match org_client.list_accounts(list_accounts_input).await {
        Ok(output) => {
            match output.accounts {
                Some(accounts_list) => {
                    let config_string = generate_aws_config(
                        "eu-west-2",
                        "json",
                        "https://synalogik-sso.awsapps.com/start",
                        "eu-west-2",
                        "AdministratorAccess",
                        &accounts_list,
                    ).await;
                    println!("{}", config_string.unwrap());
                }

                None => println!("No accounts in organization")
            }
        }
        Err(error) => {
            println!("Error: {:?}", error);
        }
    }
}

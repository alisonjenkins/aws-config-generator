use std::default::Default;
use std::io;

use tera::{Context, Tera};
use rusoto_core::Region;
use rusoto_organizations::{Account, Organizations, OrganizationsClient, ListAccountsRequest};

async fn generate_aws_config(accounts_list: Vec<Account>) -> io::Result<String> {
    let tera = match Tera::new("templates/**/*.j2") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };

    let mut context = Context::new();
    context.insert("account_profiles", accounts_list);

    Ok(config_string.to_string())
}

#[tokio::main]
async fn main() -> () {
    let org_client = OrganizationsClient::new(Region::default());
    let list_accounts_input: ListAccountsRequest = Default::default();

    match org_client.list_accounts(list_accounts_input).await {
        Ok(output) => {
            match output.accounts {
                Some(accounts_list) => {
                    let config_string = generate_aws_config(accounts_list).await;
                    println!("{:?}", config_string);
                }

                None => println!("No accounts in organization")
            }
        }
        Err(error) => {
            println!("Error: {:?}", error);
        }
    }
}

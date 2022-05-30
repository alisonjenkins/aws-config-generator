use aws_config::RetryConfig;

use aws_config_generator::configgen;
use aws_sdk_organizations::model::Account;

#[tokio::main]
async fn main() -> () {
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
            eprintln!("STS Get Caller Identity failed: {}\nUnable to identify the organisation's main account.", err);
            std::process::exit(1);
        }
    };

    let mut accounts_list = Vec::<Account>::new();
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
                    Some(mut resp_accounts) => accounts_list.append(&mut resp_accounts),
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

mod configgen;

use futures_util::stream::StreamExt;

#[tokio::main]
async fn main() {
    let _args = configgen::arg_parsing::get_args().await;
    let config = configgen::config::get_config();
    let awsconfig = aws_config::load_from_env().await;

    let org_client = aws_sdk_organizations::Client::new(&awsconfig);

    let sts_client = aws_sdk_sts::Client::new(&awsconfig);
    let org_main_account = sts_client
        .get_caller_identity()
        .send()
        .await
        .unwrap()
        .account;

    let accounts_paginator = org_client.list_accounts().into_paginator().send();
    let accounts = accounts_paginator.collect::<Result<Vec<_>, _>>().await?;
    // let paginator = client.list_tables().into_paginator().items().send();
    // let table_names = paginator.collect::<Result<Vec<_>, _>>().await?;

    let config_string = configgen::generate::generate_aws_config(
        &org_main_account.unwrap(),
        config["aws_cli_options"]["default_region"]
            .as_str()
            .unwrap(),
        config["aws_cli_options"]["default_output_type"]
            .as_str()
            .unwrap(),
        config["sso_options"]["sso_url"].as_str().unwrap(),
        config["sso_options"]["sso_region"].as_str().unwrap(),
        config["sso_options"]["sso_role"].as_str().unwrap(),
        &accounts,
    )
    .await;
    println!("{}", config_string.unwrap());
    ()
}

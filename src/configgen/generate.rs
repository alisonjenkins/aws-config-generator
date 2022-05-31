use std::io;

use aws_sdk_organizations::model::Account;
use std::collections::BTreeMap;

pub async fn generate_aws_config(
    org_main_account: &String,
    default_region: &str,
    default_output_type: &str,
    sso_start_url: &str,
    sso_region: &str,
    sso_role_name: &str,
    accounts_list: &BTreeMap<String, Account>,
    config_data: &toml::Value,
) -> io::Result<String> {
    let master_account_name = match config_data.get("account_name_overrides") {
        Some(overrides) => match overrides.get(&org_main_account) {
            Some(name) => Some(name.as_str().unwrap().to_string()),
            None => None,
        },
        None => None,
    };

    let mut config_string: String = format!(
        "[default]\nregion={}\noutput={}\n\n[profile {}]\nsso_start_url = {}\nsso_region = {}\nregion = {}\noutput = {}\nsso_account_id = {}\nsso_role_name = {}\n\n",
        &default_region,
        &default_output_type,
        &master_account_name.unwrap_or("main".to_string()),
        sso_start_url,
        sso_region,
        default_region,
        default_output_type,
        org_main_account,
        sso_role_name
    );

    for account in accounts_list.keys().into_iter() {
        let account_id = match &accounts_list[account].id {
            Some(id) => id,
            None => {
                eprintln!("No account ID!");
                std::process::exit(1);
            }
        };

        let mut account_name = match config_data.get("account_name_overrides") {
            Some(overrides) => match overrides.get(&account_id) {
                Some(name) => name.as_str().unwrap().to_string(),
                None => "".to_string(),
            },
            None => String::from(""),
        };

        if account_name == "" {
            match &accounts_list[account].name {
                Some(name) => {
                    account_name = name.clone();
                    account_name = account_name.replace(" ", "-").to_lowercase();
                }
                None => {
                    eprintln!("No account Name!");
                    std::process::exit(1);
                }
            }
        };

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

use std::io;

use aws_sdk_organizations::model::Account;
use std::borrow::Borrow;

pub async fn generate_aws_config(
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

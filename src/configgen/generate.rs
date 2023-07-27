use std::io;

use indoc::formatdoc;
use aws_sdk_organizations::types::Account;

pub struct GenerateAWSConfigInput {
    pub org_main_account: String,
    pub default_region: String,
    pub default_output_type: String,
    pub sso_start_url: String,
    pub sso_region: String,
    pub sso_role_name: String,
    pub accounts_list: Vec<Account>,
}

pub async fn generate_aws_config(
    input: GenerateAWSConfigInput
) -> io::Result<String> {
    let sso_start_url = input.sso_start_url;
    let sso_region = input.sso_region;
    let default_output_type = input.default_output_type;
    let sso_account_id = input.org_main_account;
    let sso_role_name = input.sso_role_name;
    //
    let mut config_string: String = formatdoc!(
        "[default]
        region={sso_region}
        output={default_output_type}

        [profile main]
        sso_start_url = {sso_start_url}
        sso_region = {sso_region}
        region = {sso_region}
        output = {default_output_type}
        sso_account_id = {sso_account_id}
        sso_role_name = {sso_role_name}

        "
    );

    for account in input.accounts_list {
        let account_name = if let Some(account_name) = &account.name {
                account_name.replace(' ', "-").to_lowercase()
        } else {
            panic!("An account is missing a name. All accounts must have a name to be used with the config generator.");
        };

        let Some(account_id) = account.id else {
                panic!("One of the accounts was missing the account ID. Aborting!");
        };


        config_string = config_string + &formatdoc!(
                "[profile {account_name}]
                sso_start_url = {sso_start_url}
                sso_region = {sso_region}
                region = {sso_region}
                output = {default_output_type}
                sso_account_id = {account_id}
                sso_role_name = {sso_role_name}

                [profile {account_name}-script]
                credential_process = aws-vault exec {account_name} --json
                "
        );
    }

    Ok(config_string)
}

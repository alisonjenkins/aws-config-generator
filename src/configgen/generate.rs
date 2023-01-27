use aws_sdk_organizations::model::Account;
use std::collections::BTreeMap;
use std::io;
use tera::{Context, Tera};

async fn get_account_name_tag(
    org_client: &aws_sdk_organizations::Client,
    account_id: &str,
) -> Option<String> {
    match org_client
        .list_tags_for_resource()
        .resource_id(account_id)
        .send()
        .await
        .unwrap_or_else(|_| panic!("Failed to list tags for account {account_id}"))
        .tags
    {
        Some(tags) => tags
            .iter()
            .find(|&tag| tag.key.as_ref().unwrap() == &String::from("Name"))
            .map(|tag| tag.value.as_ref().unwrap().clone()),
        None => None,
    }
}

pub async fn generate_aws_config(
    org_main_account: &String,
    default_region: &str,
    default_output_type: &str,
    sso_start_url: &str,
    sso_region: &str,
    sso_role_name: &str,
    accounts_list: &BTreeMap<String, Account>,
    config_data: &toml::Value,
    name_by_account_name_tags: bool,
    org_client: aws_sdk_organizations::Client,
    profile_template: String,
) -> io::Result<String> {
    let mut config_string = format!(
        "[default]\nregion={}\noutput={}\n\n",
        &default_region, &default_output_type
    );

    let tera = match Tera::new("templates/**/*") {
        Ok(t) => t,
        Err(e) => {
            println!("Error parsing template(s): {}", e);
            ::std::process::exit(1);
        }
    };

    for account in accounts_list.keys() {
        let account_id = if let Some(id) = &accounts_list[account].id {
            id
        } else {
            eprintln!("No account ID!");
            std::process::exit(1);
        };

        let account_name = match config_data.get("account_name_overrides") {
            Some(overrides) => overrides
                .get(account_id)
                .map(|name| name.as_str().unwrap().to_string()),
            None => None,
        };

        let account_name = if name_by_account_name_tags && account_name.is_none() {
            get_account_name_tag(&org_client, accounts_list[account].id.as_ref().unwrap()).await
        } else {
            account_name
        };

        let account_name = if account_name.is_none() {
            match &accounts_list[account].name {
                Some(name) => name.replace(' ', "-").to_lowercase(),
                None => {
                    eprintln!("No account Name!");
                    std::process::exit(1);
                }
            }
        } else {
            account_name.unwrap()
        };

        let mut context = Context::new();
        context.insert("account_id", &account_id);
        context.insert("account_name", &account_name);
        context.insert("output", &default_output_type);
        context.insert("hub_role_profile", "terraform-hub");
        context.insert("hub_role_account_id", &org_main_account);
        context.insert(
            "hub_role_arn",
            &format!(
                "arn:aws:iam::{}:role/{}",
                &org_main_account, &"terraform-hub"
            ),
        );
        context.insert("region", &default_region);
        context.insert("sso_region", &sso_region);
        context.insert("sso_role_name", &sso_role_name);
        context.insert("sso_start_url", &sso_start_url);
        config_string = config_string
            + &tera
                .render(&profile_template, &context)
                .unwrap_or_else(|_| {
                    panic!(
                        "Unable to render account profile template for account: {}",
                        account_id
                    )
                });
    }

    Ok(config_string)
}

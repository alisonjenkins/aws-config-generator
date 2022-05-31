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
        .expect(format!("Failed to list tags for account {account_id}").as_str())
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
    let master_account_name = match config_data.get("account_name_overrides") {
        Some(overrides) => match overrides.get(&org_main_account) {
            Some(name) => Some(name.as_str().unwrap().to_string()),
            None => None,
        },
        None => {
            if name_by_account_name_tags {
                match get_account_name_tag(&org_client, &org_main_account).await {
                    Some(name) => Some(name),
                    None => None,
                }
            } else {
                None
            }
        }
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


    let tera = match Tera::new("templates/**/*") {
        Ok(t) => t,
        Err(e) => {
            println!("Error parsing template(s): {}", e);
            ::std::process::exit(1);
        }
    };

    for account in accounts_list.keys().into_iter() {
        let account_id = match &accounts_list[account].id {
            Some(id) => id,
            None => {
                eprintln!("No account ID!");
                std::process::exit(1);
            }
        };

        let account_name = match config_data.get("account_name_overrides") {
            Some(overrides) => match overrides.get(&account_id) {
                Some(name) => Some(name.as_str().unwrap().to_string()),
                None => None,
            },
            None => None,
        };

        let account_name = if name_by_account_name_tags && account_name.is_none() {
            match get_account_name_tag(&org_client, &accounts_list[account].id.as_ref().unwrap())
                .await
            {
                Some(name) => Some(name),
                None => None,
            }
        } else {
            account_name
        };

        let account_name = if account_name == None {
            match &accounts_list[account].name {
                Some(name) => name.replace(" ", "-").to_lowercase(),
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
        context.insert("region", &default_region);
        context.insert("sso_region", &sso_region);
        context.insert("sso_start_url", &sso_start_url);
        context.insert("sso_role_name", &sso_role_name);
        config_string = config_string + &tera.render(&profile_template, &context).expect(format!("Unable to render account profile template for account: {}", account_id).as_str()).as_str();
    }

    Ok(config_string)
}

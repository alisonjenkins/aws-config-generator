# aws-config-generator

Generates an AWS CLI configuration file to enable assuming roles using AWS SSO for each account in an AWS Organisation.

*Note* This is still very much work in progress and there are still missing and under developed features. The ones I have planned
at the moment are in the Todo list below. If you have any features you would like to add please open a PR or an issue to get your feature
added to the Todo list or even better submit a PR implementing it or adding it to the Todo list.

## Usage

To bootstrap the config you will need to first install [aws-vault](https://github.com/99designs/aws-vault) and then create
a initial stub config file. The config file should be at the standard `~/.aws/config` location and contain the AWS SSO
config for authenticating with it to be able to query the accounts that are part of the organisation.

Here is the template for this configuration:

```
[profile main]
sso_start_url = https://my-sso.awsapps.com/start
sso_region = eu-west-2
region = eu-west-2
output = json
sso_account_id = 111111111111
sso_role_name = MyAccess
```

```
aws-vault exec main -- aws-config-generator > ~/.aws/config.generated

# Check config using:
cat ~/.aws/config.generated

# If happy with the generated config:
mv ~/.aws/config.generated ~/.aws/config
```

# Building

To build just run: `cargo build` in the project root.

# TODO List
* Implement verbosity, config path and output path CLI argument code.
* Update the code to query what permission set the user has assigned to them for each account and if not assigned a permission set do not generate a profile for that account.
* Finish implementing asdf plugin: [asdf-aws-config-generator](https://github.com/alanjjenkins/asdf-aws-config-generator)
* Create tests and document code.
* Setup dependabot to create automatic PRs when dependencies are updated (needs automated tests first).
* Profile name aliases overrides (allow overriding the names to generate the account profiles with).
* Allow setting multiple master account profile names to iterate over for companies that have multiple Organisations.

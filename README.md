# aws-config-generator

Generates an AWS config for Synalogik users to use the AWS CLI and other command line tools with AWS SSO for each account in the Synalogik organisation.

This tool was created to ensure that we can easily create the configurations for these accounts without risk of human error which could result in big problems (e.g. aws-nuke getting ran in the wrong account would be *bad*).


# TODO List
* Figure out why the syn-master account is not part of the list and if not returned by the organisation API have it automatically added to the config.
* Update the code to query what role the user has assigned to them for each account and if not assigned a role do not generate a profile for that account.
* Sorting of the profiles alphabetically by name.
* Profile name aliases (allow providing alternative names to generate the config profiles with (some are not named in a nice way e.g. ci/cd.

use clap::{
    App,
    // SubCommand
    // Arg,
    ArgMatches,
};

pub async fn get_args<'a>() -> Result<ArgMatches<'a>, String> {
    let matches = App::new("aws-config-generator")
                          .version("AWS_CONFIG_GENERATOR_VERSION")
                          .author("Alan Jenkins <alan.james.jenkins@gmail.com>")
                          .about("Generates AWS CLI configs for SSO authentication from your AWS Organisations accounts.")
                          // .arg(Arg::with_name("config")
                          //      .short("c")
                          //      .long("config")
                          //      .value_name("FILE")
                          //      .help("Sets a custom config file")
                          //      .takes_value(true))
                          // .arg(Arg::with_name("v")
                          //      .short("v")
                          //      .multiple(true)
                          //      .help("Sets the level of verbosity"))
                          .get_matches();

    // Gets a value for config if supplied by user, or defaults to "default.conf"
    // let config = matches.value_of("config").unwrap_or("config.toml");
    // println!("Value for config: {}", config);

    // Vary the output based on how many times the user used the "verbose" flag
    // (i.e. 'myprog -v -v -v' or 'myprog -vvv' vs 'myprog -v'
    // match matches.occurrences_of("v") {
    //     0 => println!("No verbose info"),
    //     1 => println!("Some verbose info"),
    //     2 => println!("Tons of verbose info"),
    //     3 | _ => println!("Don't be crazy"),
    // }
    Ok(matches)
}

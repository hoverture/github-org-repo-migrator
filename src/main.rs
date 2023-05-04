mod api;
mod errors;
mod git;

use api::process_repositories;
use clap::{App, Arg};

#[tokio::main]
async fn main() {
    let matches = App::new("Github Org Repo Migrator")
        .version("0.1.0")
        .author("Aingaran <aingaran.elango@hoverture.com>")
        .about("Copies repositories from one GitHub organization to another")
        .arg(
            Arg::new("token")
                .short('t')
                .long("token")
                .value_name("TOKEN")
                .help("Sets your GitHub personal access token")
                .required(true),
        )
        .arg(
            Arg::new("source")
                .short('s')
                .long("source")
                .value_name("SOURCE_ORG")
                .help("Sets the source organization")
                .required(true),
        )
        .arg(
            Arg::new("destination")
                .short('d')
                .long("destination")
                .value_name("DEST_ORG")
                .help("Sets the destination organization")
                .required(true),
        )
        .arg(
            Arg::new("skip_forks")
                .short('f')
                .long("skip-forks")
                .help("Skip forked repositories"),
        )
        .arg(
            Arg::new("topics")
                .short('p')
                .long("topics")
                .value_name("TOPICS")
                .help("Comma-separated list of topics to add to the migrated repositories")
                .takes_value(true),
        )
        .get_matches();

    let token = matches.value_of("token").unwrap();
    let source_org = matches.value_of("source").unwrap();
    let target_org = matches.value_of("destination").unwrap();
    let skip_forks = matches.is_present("skip_forks");
    let topics = matches
        .value_of("topics")
        .map(|t| t.split(',').map(String::from).collect())
        .unwrap_or_else(|| vec![]);

    match process_repositories(token, source_org, target_org, skip_forks, topics).await {
        Ok(_) => println!("Repositories successfully copied"),
        Err(e) => eprintln!("Error occurred: {}", e),
    }
}

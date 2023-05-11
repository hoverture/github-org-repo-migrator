mod api;
mod errors;
mod git;

use api::process_repositories;
use clap::{App, Arg};
use log::LevelFilter;

#[tokio::main]
async fn main() {
    env_logger::Builder::new()
        .filter(None, LevelFilter::Info)
        .init();

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
                .short('o')
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
                .short('s')
                .long("skip-forks")
                .help("Skip forked repositories"),
        )
        .arg(
            Arg::new("force_update")
                .short('f')
                .long("force-update")
                .help("force update repositories (if exists)"),
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
    let force_update = matches.is_present("force_update");
    let topics = matches
        .value_of("topics")
        .map(|t| t.split(',').map(String::from).collect())
        .unwrap_or_else(|| vec![]);

    match process_repositories(
        token,
        source_org,
        target_org,
        skip_forks,
        force_update,
        topics,
    )
    .await
    {
        Ok(r) => {
            if r.is_empty() {
                log::info!("Repositories successfully copied");
            } else {
                log::warn!("Repositories partially copied... \nFailed repositories: {r:#?}")
            }
        }
        Err(e) => log::error!("{e:#?}"),
    }
}

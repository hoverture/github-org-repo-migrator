use crate::errors::RepoCopyError;
use crate::git::process_repository;
use reqwest::header::{self, HeaderValue};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Repo {
    pub name: String,
    pub full_name: String,
    pub clone_url: String,
    pub description: Option<String>,
    pub private: bool,
    pub fork: bool,
    pub url: String,
}

pub async fn process_repositories(
    token: &str,
    source_org: &str,
    target_org: &str,
    skip_forks: bool,
    force_update: bool,
    topics: Vec<String>,
) -> Result<Vec<String>, RepoCopyError> {
    let mut failed_repos: Vec<String> = Vec::new();
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        HeaderValue::from_str(&format!("token {}", token))?,
    );
    headers.insert(header::ACCEPT, "application/vnd.github+json".parse()?);
    headers.insert(header::USER_AGENT, "github-org-repo-migrator".parse()?);

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    let mut page = 1;
    let mut repos: Vec<Repo> = vec![];

    loop {
        let url = format!(
            "https://api.github.com/orgs/{}/repos?per_page=100&page={}",
            source_org, page
        );
        let response: Vec<Repo> = client.get(&url).send().await?.json().await?;
        if response.is_empty() {
            break;
        }

        repos.extend(response);
        page += 1;
    }
    let repo_count = repos.len();
    log::info!("found {repo_count} repositories in {source_org} organisation");

    for repo in repos {
        if skip_forks && repo.fork {
            log::info!("Skipping forked repository: {}", repo.name);
            continue;
        }

        log::info!("Processing repository: {}", repo.name);
        match process_repository(
            &client,
            &repo.name,
            &repo.clone_url,
            &repo.description,
            repo.private,
            topics.clone(),
            token,
            target_org,
            force_update,
        )
        .await
        {
            Ok(_) => {
                let name = repo.name;
                log::info!("Processed successfully: {name}");
            },
            Err(e) => {
                log::error!("{e:#?}");
                failed_repos.push(repo.name.clone())
            }
        };
    }

    Ok(failed_repos)
}

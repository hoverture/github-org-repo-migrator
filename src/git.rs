use crate::api::Repo;
use crate::errors::RepoCopyError;
use git2::RemoteCallbacks;
use std::collections::HashMap;

pub async fn process_repository(
    client: &reqwest::Client,
    repo_name: &str,
    clone_url: &str,
    description: &Option<String>,
    private: bool,
    topics: Vec<String>,
    token: &str,
    target_org: &str,
    force_update: bool,
) -> Result<(), RepoCopyError> {
    let repo_check_url = format!("https://api.github.com/repos/{}/{}", target_org, repo_name);

    let repo_check_response = client.get(&repo_check_url).send().await;

    let mut new_repo: Option<Repo> = None;

    if let Ok(response) = repo_check_response {
        if response.status().is_success() {
            if force_update {
                log::info!(
                    "Repository already exists in the destination organization: {}. Updating...",
                    repo_name
                );
                new_repo = response.json().await.unwrap();
            } else {
                log::info!(
                    "Repository already exists in the destination organization: {}",
                    repo_name
                );
                return Ok(());
            }
        }
    }

    let repo_temp_dir = format!("{}_temp", repo_name);
    let clone_dir = tempfile::Builder::new().prefix(&repo_temp_dir).tempdir()?;
    let clone_dir_path = clone_dir.path();

    log::debug!("cloning {clone_url:#?} into {clone_dir_path:#?}");

    let mut fetch_options = git2::FetchOptions::new();
    fetch_options.remote_callbacks(get_callbacks(token.to_string().clone()).await);

    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(fetch_options);
    let repo = builder.clone(
        &format!(
            "https://x-access-token:{}@github.com/{}",
            token,
            clone_url.trim_start_matches("https://github.com/")
        ),
        clone_dir.path(),
    )?;

    // Update the remote configuration to fetch all branches
    repo.remote_add_fetch("origin", "+refs/heads/*:refs/remotes/origin/*")?;

    let mut fetch_options2 = git2::FetchOptions::new();
    fetch_options2.remote_callbacks(get_callbacks(token.to_string().clone()).await);

    // Fetch all branches from the remote
    repo.find_remote("origin")?.fetch(
        &["+refs/heads/*:refs/remotes/origin/*"],
        Some(&mut fetch_options2),
        None,
    )?;

    log::debug!("checking out all the branches locally");

    // Checkout all branches locally
    let branches = repo.branches(Some(git2::BranchType::Remote))?;
    for branch_result in branches {
        let (branch, _branch_type) = branch_result?;
        let reference = branch.into_reference();
        let branch_name = reference.shorthand().ok_or_else(|| {
            RepoCopyError::Git(git2::Error::from_str("Error getting branch name"))
        })?;

        if branch_name.starts_with("origin/HEAD") {
            continue;
        }

        let local_branch_name = branch_name.strip_prefix("origin/").unwrap();
        let mut local_branch = match repo.find_branch(local_branch_name, git2::BranchType::Local) {
            Ok(branch) => {
                let reference_name = format!("refs/heads/{}", local_branch_name);
                if let Ok(mut reference) = repo.find_reference(&reference_name) {
                    reference.set_target(reference.target().unwrap(), "Update branch")?;
                }
                branch
            }
            Err(_) => repo.branch(local_branch_name, &reference.peel_to_commit()?, false)?,
        };

        local_branch.set_upstream(Some(branch_name))?;
    }

    if new_repo.is_none() {
        new_repo = Some(
            create_github_repository(client, repo_name, description, private, target_org).await?,
        );
    }

    let new_repo = new_repo.unwrap();

    set_repository_topics(client, &new_repo.url, topics).await?;

    let remote_url = format!(
        "https://x-access-token:{}@github.com/{}",
        token, new_repo.full_name
    );
    let mut new_remote = repo.remote("new-origin", &remote_url)?;
    let mut push_options = git2::PushOptions::new();
    push_options.remote_callbacks(get_callbacks(token.to_string().clone()).await);

    let branches = repo.branches(Some(git2::BranchType::Local))?;
    let mut refspecs = Vec::new();
    let mut batch_size = 0;

    for branch in branches {
        let (branch, _branch_type) = branch?;
        let name = branch.get().name().ok_or_else(|| {
            RepoCopyError::Git(git2::Error::from_str("Error getting branch name"))
        })?;

        refspecs.push(format!("+{}:{}", name, name));
        batch_size += 1;

        if batch_size >= 5 {
            log::debug!("current batch has {batch_size} branches, pushing to new remote");
            new_remote.push(&refspecs, Some(&mut push_options))?;
            refspecs.clear();
            batch_size = 0;
        }
    }

    if !refspecs.is_empty() {
        log::debug!("pushing remaining branches to the new remote");
        new_remote.push(&refspecs, Some(&mut push_options))?;
    }

    Ok(())
}

async fn create_github_repository(
    client: &reqwest::Client,
    repo_name: &str,
    description: &Option<String>,
    private: bool,
    target_org: &str,
) -> Result<Repo, RepoCopyError> {
    let create_repo_url = format!("https://api.github.com/orgs/{}/repos", target_org);

    let private_str = private.to_string();
    let mut payload = HashMap::new();
    payload.insert("name", repo_name);
    payload.insert("private", &private_str);
    if let Some(desc) = description {
        payload.insert("description", desc);
    }
    payload.insert("is_template", "false");

    let response = client.post(&create_repo_url).json(&payload).send().await?;

    let status = response.status();
    if !status.is_success() {
        return Err(RepoCopyError::CreateRepoError(status));
    }

    let repo: Repo = response.json().await?;
    Ok(repo)
}

async fn get_callbacks(token: String) -> RemoteCallbacks<'static> {
    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(move |_url, _username_from_url, _allowed_types| {
        git2::Cred::userpass_plaintext(&token, "")
    });
    callbacks
}

async fn set_repository_topics(
    client: &reqwest::Client,
    repo_url: &str,
    topics: Vec<String>,
) -> Result<(), RepoCopyError> {
    if topics.is_empty() {
        log::debug!("No topic provided to add for the repo: {}", repo_url);
        return Ok(());
    }

    let topics_url = format!("{}/topics", repo_url);
    let mut payload = HashMap::new();
    payload.insert("names", topics);

    let response = client.put(&topics_url).json(&payload).send().await?;

    let status = response.status();
    if !status.is_success() {
        return Err(RepoCopyError::SetTopicsError(status));
    }

    Ok(())
}

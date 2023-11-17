use std::{
    env,
    path::{Path, PathBuf},
    str::FromStr,
};

use git2::{build::RepoBuilder, Cred, RemoteCallbacks};

use crate::error::ApiResult;

const GIT_CLONE_FOLDER: &str = "/tmp/";

fn repo_builder<'a>() -> RepoBuilder<'a> {
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        Cred::ssh_key(
            username_from_url.unwrap(),
            None,
            Path::new(
                &env::var("PRIVATE_SSH_KEY")
                    .unwrap_or_else(|_| format!("{}/.ssh/id_rsa", env::var("HOME").unwrap())),
            ),
            env::var("PASSPHRASE").as_deref().ok(),
        )
    });
    let mut fo = git2::FetchOptions::new();
    fo.remote_callbacks(callbacks);

    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(fo);
    builder
}

pub fn pull_or_clone(url: &str) -> ApiResult<String> {
    let mut repo_builder = repo_builder();

    let repository_hex_name = hex::encode(url);
    // safety we expect /tmp to be valid. won't work for windows
    let mut repository_path = PathBuf::from_str(GIT_CLONE_FOLDER).unwrap();
    repository_path.push(&repository_hex_name);
    if repository_path.exists() {
        todo!();
        // let repo = Repository::open(repository_path)?;
        // repo.checkout_head(None)?;
    } else {
        repo_builder.clone(url, &repository_path)?;
    }
    Ok(repository_hex_name)
}

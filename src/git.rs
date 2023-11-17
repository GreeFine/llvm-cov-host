use std::{env, path::PathBuf, str::FromStr};

use git2::{Cred, FetchOptions, RemoteCallbacks, Repository};

use crate::error::ApiResult;

const GIT_CLONE_FOLDER: &str = "/tmp/";

fn fetch_options<'a>() -> FetchOptions<'a> {
    let mut callbacks = RemoteCallbacks::new();
    let priv_key_path = PathBuf::from_str(
        &env::var("PRIVATE_SSH_KEY")
            .unwrap_or_else(|_| format!("{}/.ssh/id_ed25519", env::var("HOME").unwrap())),
    )
    .unwrap();
    let pub_key_path = priv_key_path.clone().with_extension("pub");
    callbacks.credentials(move |_url, username_from_url, _allowed_types| {
        Cred::ssh_key(
            username_from_url.unwrap(),
            Some(&pub_key_path),
            &priv_key_path,
            env::var("PASSPHRASE").as_deref().ok(),
        )
    });
    let mut fo = git2::FetchOptions::new();
    fo.remote_callbacks(callbacks);

    fo
}

pub fn pull_or_clone(url: &str, branch: &str) -> ApiResult<PathBuf> {
    let repository_hex_name = hex::encode(url);
    // safety we expect /tmp to be valid. won't work for windows
    let mut repository_path = PathBuf::from_str(GIT_CLONE_FOLDER).unwrap();
    repository_path.push(&repository_hex_name);

    if repository_path.exists() {
        let repo: Repository = Repository::open(&repository_path)?;
        let (object, reference) = repo.revparse_ext(branch).expect("Object not found");
        repo.checkout_tree(&object, None)
            .expect("Failed to checkout");
        repo.set_head(reference.unwrap().name().unwrap())
            .expect("setting head");

        let mut fo = fetch_options();
        let mut remote = repo.find_remote("origin").expect("default remote origin");
        fo.download_tags(git2::AutotagOption::All);
        remote.fetch(&[branch], Some(&mut fo), None)?;
    } else {
        let fo = fetch_options();
        let mut cloner = git2::build::RepoBuilder::new();
        cloner.fetch_options(fo);
        cloner.branch(branch);
        cloner.clone(url, &repository_path)?;
    }
    Ok(repository_path)
}

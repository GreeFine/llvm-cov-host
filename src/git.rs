use std::{env, fs, path::PathBuf, str::FromStr};

use git2::{Cred, CredentialType, FetchOptions, RemoteCallbacks, Repository};

use crate::error::ApiResult;

/// Get path from ENV key SSH_KEY_PATH or default to id_ed25519 in the home .ssh directory
pub fn get_ssh_key_path() -> PathBuf {
    PathBuf::from_str(
        &env::var("SSH_KEY_PATH")
            .unwrap_or_else(|_| format!("{}/.ssh/id_ed25519", env::var("HOME").unwrap())),
    )
    .unwrap()
}

fn create_fetch_options<'a>() -> FetchOptions<'a> {
    let mut fo = git2::FetchOptions::new();
    let priv_key_path = get_ssh_key_path();
    if !priv_key_path.exists() {
        return fo;
    };

    let pub_key_path = priv_key_path.with_extension("pub");

    let mut credential_tries = 0;
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(move |_url, username_from_url, allowed_types| {
        if !allowed_types.contains(CredentialType::SSH_KEY) {
            return Err(git2::Error::from_str(
                "Git server doesn't allow CredentialType::SSH_KEY",
            ));
        }
        if credential_tries >= 3 {
            return Err(git2::Error::from_str(
                "unable to authenticate with credentials after 3 tries",
            ));
        }
        credential_tries += 1;

        Cred::ssh_key(
            username_from_url.unwrap(),
            Some(&pub_key_path),
            &priv_key_path,
            env::var("SSH_KEY_PASSPHRASE").as_deref().ok(),
        )
    });
    fo.remote_callbacks(callbacks);
    fo
}

/// Clone the repository, or pull if it already exist, [create_fetch_options] is used to provide authentication.
pub fn pull_or_clone(url: &str, branch: &str) -> ApiResult<PathBuf> {
    let repository_hex_name = hex::encode(url);
    // safety we expect /tmp to be valid. won't work for windows
    let repository_path = PathBuf::from_str(crate::REPOSITORIES_DIR)
        .unwrap()
        .join(repository_hex_name);

    let mut fo = create_fetch_options();
    if repository_path.exists() && repository_path.read_dir()?.next().is_some() {
        let repo: Repository = Repository::open(&repository_path)?;
        let (object, reference) = repo.revparse_ext(branch).expect("Object not found");
        repo.checkout_tree(&object, None)
            .expect("Failed to checkout");
        repo.set_head(reference.unwrap().name().unwrap())
            .expect("setting head");

        let mut remote = repo.find_remote("origin").expect("default remote origin");
        fo.download_tags(git2::AutotagOption::All);
        remote.fetch(&[branch], Some(&mut fo), None)?;
    } else {
        let mut cloner = git2::build::RepoBuilder::new();
        cloner.fetch_options(fo);
        cloner.branch(branch);
        if let Err(error) = cloner.clone(url, &repository_path) {
            if repository_path.exists() {
                fs::remove_dir_all(&repository_path).expect("removing git dir after failed clone");
            }
            Err(error)?;
        }
    }
    Ok(repository_path)
}

use std::{env, fs, path::PathBuf, str::FromStr};

use git2::{Cred, CredentialType, FetchOptions, RemoteCallbacks, Repository};

use crate::error::ApiResult;

fn fetch_options<'a>() -> FetchOptions<'a> {
    let mut callbacks = RemoteCallbacks::new();
    let priv_key_path = PathBuf::from_str(
        &env::var("SSH_KEY_PATH")
            .unwrap_or_else(|_| format!("{}/.ssh/id_ed25519", env::var("HOME").unwrap())),
    )
    .unwrap();
    let priv_key = fs::read_to_string(&priv_key_path).unwrap();
    let pub_key_path = priv_key_path.with_extension("pub");
    let pub_key = fs::read_to_string(pub_key_path).unwrap();

    let mut credential_tries = 0;
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

        Cred::ssh_key_from_memory(
            username_from_url.unwrap(),
            Some(&pub_key),
            &priv_key,
            env::var("SSH_KEY_PASSPHRASE").as_deref().ok(),
        )
    });
    let mut fo = git2::FetchOptions::new();
    fo.remote_callbacks(callbacks);

    fo
}

pub fn pull_or_clone(url: &str, branch: &str) -> ApiResult<PathBuf> {
    let repository_hex_name = hex::encode(url);
    // safety we expect /tmp to be valid. won't work for windows
    let repository_path = PathBuf::from_str(crate::REPOSITORIES_DIR)
        .unwrap()
        .join(repository_hex_name);

    if repository_path.exists() && repository_path.read_dir()?.next().is_some() {
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
        if let Err(error) = cloner.clone(url, &repository_path) {
            if repository_path.exists() {
                fs::remove_dir_all(&repository_path).expect("removing git dir after failed clone");
            }
            Err(error)?;
        }
    }
    Ok(repository_path)
}

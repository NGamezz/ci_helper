use crate::cleaner::{args::CleanArgs, error::CleanError};
use std::{path::Path, sync::Arc};

use ignore::{WalkBuilder, WalkState, gitignore::Gitignore};

pub fn ignore_from_args(args: CleanArgs) -> Option<ignore::gitignore::Gitignore> {
    let ignore_options = &args.ignore_settings;

    let path = Path::new(&ignore_options.ignore_path);

    if path.exists() {
        let (ignore, error) = Gitignore::new(path);

        if let Some(err) = error {
            panic!("{err:?}");
        }

        return Some(ignore);
    }

    None
}

pub async fn process_clean_command(args: CleanArgs) -> Result<(), CleanError> {
    let ignore = ignore_from_args(args).ok_or(CleanError::IgnoreFileNotFound)?;
    let ignore = Arc::new(ignore);

    let cwd = std::env::current_dir()?;

    let walker = WalkBuilder::new(cwd)
        .ignore(false)
        .git_ignore(false)
        .hidden(true)
        .threads(num_cpus::get())
        .build_parallel();

    let (tx, mut rx) = tokio::sync::mpsc::channel(32);

    rayon::spawn(move || {
        walker.run(|| {
            let cl = &ignore;
            let tx = tx.clone();

            Box::new(move |result| {
                let cl = cl.clone();
                let tx = tx.clone();

                let entry = match result {
                    Ok(entry) => entry,
                    Err(_) => return WalkState::Skip,
                };

                let path = entry.path();
                let is_dir = path.is_dir();

                let matches = cl.as_ref().matched(path, is_dir);

                if matches.is_ignore() {
                    let result = tx.blocking_send(path.to_owned());

                    if result.is_ok() {
                        return WalkState::Skip;
                    }
                }

                WalkState::Continue
            })
        });
        drop(tx);
    });

    let mut set = tokio::task::JoinSet::new();

    while let Some(path) = rx.recv().await {
        set.spawn(async move {
            let is_dir = path.is_dir();

            let result = match is_dir {
                true => tokio::fs::remove_dir_all(&path).await,
                false => tokio::fs::remove_file(&path).await,
            };

            if let Err(err) = result {
                eprintln!("Err : {err:?}");
            }
        });
    }

    set.join_all().await;

    Ok(())
}

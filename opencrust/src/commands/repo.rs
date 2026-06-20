use crate::cli::RepoCommands;
use crate::error::Result;
use crate::multi_repo::MultiRepoManager;

pub async fn handle_repo(cmd: RepoCommands) -> Result<()> {
    let repo_mgr = MultiRepoManager::new();
    match cmd {
        RepoCommands::List => {
            let repos = repo_mgr.list().await;
            if repos.is_empty() {
                println!("No repositories registered.");
                println!("\nUse 'opencrust repo add <name> <path>' to register one.");
            } else {
                println!("{:<20} {:<20} {:<30} Remote", "Name", "Branch", "Path");
                println!("{}", "-".repeat(100));
                for repo in &repos {
                    let branch = repo.branch.as_deref().unwrap_or("(detached)");
                    let remote = repo.remote.as_deref().unwrap_or("-");
                    let path_str = repo.path.display().to_string();
                    let path_short = if path_str.len() > 28 {
                        format!("...{}", &path_str[path_str.len().saturating_sub(25)..])
                    } else {
                        path_str
                    };
                    println!(
                        "{:<20} {:<20} {:<30} {}",
                        repo.name, branch, path_short, remote
                    );
                }
            }
        }
        RepoCommands::Show { name } => match repo_mgr.get(&name).await {
            Some(repo) => {
                println!("Name:       {}", repo.name);
                println!("Path:       {}", repo.path.display());
                println!(
                    "Branch:     {}",
                    repo.branch.as_deref().unwrap_or("(detached)")
                );
                println!("Remote:     {}", repo.remote.as_deref().unwrap_or("(none)"));
                println!("Tags:       {}", repo.tags.join(", "));
                println!("Registered: {}", repo.registered_at);
                if let Some(idx) = repo.last_indexed {
                    println!("Indexed:    {}", idx);
                }
            }
            None => eprintln!("Repository '{}' not found.", name),
        },
        RepoCommands::Add { name, path, tags } => {
            let tags: Vec<String> = tags
                .as_ref()
                .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default();
            match repo_mgr
                .add(name.clone(), std::path::PathBuf::from(&path), tags)
                .await
            {
                Ok(repo) => {
                    println!(
                        "Repository '{}' registered at {}",
                        repo.name,
                        repo.path.display()
                    );
                    println!(
                        "  Branch: {}",
                        repo.branch.as_deref().unwrap_or("(detached)")
                    );
                    if let Some(remote) = &repo.remote {
                        println!("  Remote: {}", remote);
                    }
                }
                Err(e) => eprintln!("Error adding repository: {}", e),
            }
        }
        RepoCommands::Remove { name } => {
            if repo_mgr.remove(&name).await {
                println!("Repository '{}' removed.", name);
            } else {
                eprintln!("Repository '{}' not found.", name);
            }
        }
        RepoCommands::Stats => {
            let stats = repo_mgr.stats().await;
            let repos = repo_mgr.list().await;
            println!("{}", stats);
            for repo in &repos {
                println!("  {}", repo.summary());
            }
        }
        RepoCommands::Git { args } => {
            let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            let results = repo_mgr.git_command_all(&args_refs).await;
            if results.is_empty() {
                println!("No repositories to run command on.");
            } else {
                for (repo, result) in &results {
                    println!("\n=== {} ({}) ===", repo.name, repo.path.display());
                    match result {
                        Ok(output) => println!("{}", output),
                        Err(e) => eprintln!("Error: {}", e),
                    }
                }
            }
        }
        RepoCommands::Search { pattern } => {
            let results = repo_mgr.search_files(&pattern).await;
            if results.is_empty() {
                println!("No matches found for pattern '{}'", pattern);
            } else {
                println!("Matches for '{}':", pattern);
                for (repo, matches) in &results {
                    println!("\n  {}:", repo.name);
                    for m in matches {
                        println!("    {}", m);
                    }
                }
            }
        }
        RepoCommands::Refresh => {
            repo_mgr.refresh_all().await;
            let repos = repo_mgr.list().await;
            println!("Refreshed {} repositories:", repos.len());
            for repo in &repos {
                println!("  {}", repo.summary());
            }
        }
    }
    Ok(())
}

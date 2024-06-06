use git2::{Cred, RemoteCallbacks, Repository};
use std::fs::{OpenOptions, File};
use std::io::{self, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::thread;
use colored::*;

fn main() {
    let repo_path = Path::new(".");
    let repo = Arc::new(Mutex::new(Repository::open(repo_path).expect("Could not open repository")));

    // Show overview of the current state of the repository
    let repo_guard = repo.lock().unwrap();
    let mut revwalk = repo_guard.revwalk().expect("Could not get revwalk");
    revwalk.push_head().expect("Could not push head");
    let current_commit_count = revwalk.count();
    
    println!("{}", "================== Repository Overview ==================".bold().underline());
    println!("{}: {}", "Current branch".bold(), repo_guard.head().unwrap().shorthand().unwrap_or("unknown"));
    println!("{}: {}", "Current commit count".bold(), current_commit_count);
    println!("{}", "=========================================================".bold().underline());

    drop(repo_guard); // Release the lock on the repo_guard

    // Ask the user for the total commit count they want to achieve
    println!("{}", "Enter the total commit count you want to achieve in this run:".bold().blue());
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read input");
    let goal_total_commits: usize = input.trim().parse().expect("Invalid input");

    if current_commit_count >= goal_total_commits {
        println!(
            "{}: The repository already has {} commits, which is equal to or exceeds the goal of {} commits.",
            "Warning".bold().yellow(),
            current_commit_count,
            goal_total_commits
        );
        return;
    }

    let readme_path = repo_path.join("README.md");
    let commit_count = Arc::new(Mutex::new(current_commit_count + 1));
    let goal_total_commits = Arc::new(goal_total_commits);
    let repo = Arc::clone(&repo);

    let writer_thread = {
        let readme_path = readme_path.clone();
        let commit_count = Arc::clone(&commit_count);
        let goal_total_commits = Arc::clone(&goal_total_commits);
        let repo = Arc::clone(&repo);
    
        thread::spawn(move || {
            while *commit_count.lock().unwrap() <= *goal_total_commits {
                // Append 10 slashes to README.md
                for _ in 0..10 {
                    if *commit_count.lock().unwrap() > *goal_total_commits {
                        break;
                    }
    
                    let mut readme_file = OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&readme_path)
                        .expect("Could not open README.md");
    
                    writeln!(readme_file, "/").expect("Could not write to README.md");
    
                    let mut repo_guard = repo.lock().unwrap();
                    let mut index = repo_guard.index().expect("Could not get repository index");
    
                    if Path::new("README.md").exists() {
                        index.add_path(Path::new("README.md")).expect("Could not add file to index");
                    }
    
                    let tree_id = index.write_tree().expect("Could not write tree");
                    let tree = repo_guard.find_tree(tree_id).expect("Could not find tree");
    
                    let parent_commit = repo_guard.head().expect("Could not get head").peel_to_commit().expect("Could not peel to commit");
    
                    let sig = repo_guard.signature().expect("Could not get signature");
                    repo_guard.commit(
                        Some("HEAD"),
                        &sig,
                        &sig,
                        &format!(
                            "Commit {}",
                            *commit_count.lock().unwrap()
                        ),
                        &tree,
                        &[&parent_commit],
                    )
                    .expect("Could not commit");
    
                    *commit_count.lock().unwrap() += 1;
                }
    
                // Empty the contents of README.md
                let readme_file = File::create(&readme_path).expect("Could not create README.md");
                readme_file.set_len(0).expect("Could not truncate README.md");
    
                *commit_count.lock().unwrap() += 1;
            }
        })
    };

    let commit_thread = {
        let repo = Arc::clone(&repo);
        let commit_count = Arc::clone(&commit_count);
        let goal_total_commits = Arc::clone(&goal_total_commits);

        thread::spawn(move || {
            while *commit_count.lock().unwrap() <= *goal_total_commits {
                // Add and commit changes
                if *commit_count.lock().unwrap() > *goal_total_commits {
                    break;
                }

                let mut repo_guard = repo.lock().unwrap();
                let mut index = repo_guard.index().expect("Could not get repository index");

                if Path::new("README.md").exists() {
                    index.add_path(Path::new("README.md")).expect("Could not add file to index");
                }

                let tree_id = index.write_tree().expect("Could not write tree");
                let tree = repo_guard.find_tree(tree_id).expect("Could not find tree");

                let parent_commit = repo_guard.head().expect("Could not get head").peel_to_commit().expect("Could not peel to commit");

                let sig = repo_guard.signature().expect("Could not get signature");
                repo_guard.commit(
                    Some("HEAD"),
                    &sig,
                    &sig,
                    &format!(
                        "Commit {}",
                        *commit_count.lock().unwrap()
                    ),
                    &tree,
                    &[&parent_commit],
                )
                .expect("Could not commit");

                *commit_count.lock().unwrap() += 1;
            }
        })
    };

    let progress_thread = {
        let commit_count = Arc::clone(&commit_count);
        let goal_total_commits = Arc::clone(&goal_total_commits);
    
        thread::spawn(move || {
            let spinner_chars = vec!['|', '/', '-', '\\'];
            let mut spinner_index = 0;
    
            while *commit_count.lock().unwrap() <= *goal_total_commits {
                let count = *commit_count.lock().unwrap();
                print!(
                    "\r{} {}: {}/{}",
                    spinner_chars[spinner_index].to_string().cyan(),
                    "Progress".bold().green(),
                    count,
                    *goal_total_commits
                );
                io::stdout().flush().unwrap();
                spinner_index = (spinner_index + 1) % spinner_chars.len();
                thread::sleep(Duration::from_millis(50));
            }
        })
    };

    writer_thread.join().expect("Writer thread panicked");
    commit_thread.join().expect("Commit thread panicked");
    progress_thread.join().expect("Progress thread panicked");

    // Set up authentication callback for push
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        Cred::ssh_key(
            username_from_url.unwrap(),
            None,
            std::path::Path::new("./id_rsa"), // Assuming SSH key is in the root of the repo
            None,
        )
    });

    let repo_guard = repo.lock().unwrap();
    let mut remote = repo_guard.find_remote("origin").expect("Could not find remote");
    let mut push_options = git2::PushOptions::new();
    push_options.remote_callbacks(callbacks);

    remote
        .push(&["refs/heads/main:refs/heads/main"], Some(&mut push_options))
        .expect("Could not push to remote");

    println!(
        "\n{}: Completed {} iterations and pushed to remote repository.",
        "Success".bold().green(),
        *goal_total_commits - current_commit_count
    );
}
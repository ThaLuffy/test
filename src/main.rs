use git2::{Cred, RemoteCallbacks, Repository};
use std::fs::{OpenOptions, File};
use std::io::{self, Write};
use std::path::Path;
use std::thread;
use std::time::Duration;
use colored::*;

fn main() {
    let repo_path = Path::new(".");
    let repo = Repository::open(repo_path).expect("Could not open repository");

    // Show overview of the current state of the repository
    let mut revwalk = repo.revwalk().expect("Could not get revwalk");
    revwalk.push_head().expect("Could not push head");
    let current_commit_count = revwalk.count();
    
    println!("{}", "================== Repository Overview ==================".bold().underline());
    println!("{}: {}", "Current branch".bold(), repo.head().unwrap().shorthand().unwrap_or("unknown"));
    println!("{}: {}", "Current commit count".bold(), current_commit_count);
    println!("{}", "=========================================================".bold().underline());

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
    let mut commit_count = current_commit_count + 1;

    let spinner_chars = vec!['|', '/', '-', '\\'];
    let mut spinner_index = 0;

    while commit_count <= goal_total_commits {
        // Append 10 slashes to README.md and commit each addition
        for _ in 0..10 {
            if commit_count > goal_total_commits {
                break;
            }

            let mut readme_file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&readme_path)
                .expect("Could not open README.md");

            writeln!(readme_file, "/").expect("Could not write to README.md");

            let mut index = repo.index().expect("Could not get repository index");
            index.add_path(Path::new("README.md")).expect("Could not add file to index");
            index.write().expect("Could not write index");

            let tree_id = index.write_tree().expect("Could not write tree");
            let tree = repo.find_tree(tree_id).expect("Could not find tree");

            let head = repo.head().expect("Could not get head");
            let parent_commit = head.peel_to_commit().expect("Could not peel to commit");

            let sig = repo.signature().expect("Could not get signature");
            repo.commit(
                Some("HEAD"),
                &sig,
                &sig,
                &format!(
                    "Add '/' character - iteration {} - total commits {}",
                    commit_count, commit_count
                ),
                &tree,
                &[&parent_commit],
            )
            .expect("Could not commit");

            commit_count += 1;

            // Update progress
            print!(
                "\r{} {}: {}/{}",
                spinner_chars[spinner_index].to_string().cyan(),
                "Progress".bold().green(),
                commit_count,
                goal_total_commits
            );
            io::stdout().flush().unwrap();
            spinner_index = (spinner_index + 1) % spinner_chars.len();
            thread::sleep(Duration::from_millis(100));
        }

        // Empty the contents of README.md and commit
        let readme_file = File::create(&readme_path).expect("Could not create README.md");
        readme_file.set_len(0).expect("Could not truncate README.md");

        let mut index = repo.index().expect("Could not get repository index");
        index.add_path(Path::new("README.md")).expect("Could not add file to index");
        index.write().expect("Could not write index");

        let tree_id = index.write_tree().expect("Could not write tree");
        let tree = repo.find_tree(tree_id).expect("Could not find tree");

        let head = repo.head().expect("Could not get head");
        let parent_commit = head.peel_to_commit().expect("Could not peel to commit");

        let sig = repo.signature().expect("Could not get signature");
        repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            &format!("Empty file - total commits {}", commit_count),
            &tree,
            &[&parent_commit],
        )
        .expect("Could not commit");

        commit_count += 1;

        // Update progress
        print!(
            "\r{} {}: {}/{}",
            spinner_chars[spinner_index].to_string().cyan(),
            "Progress".bold().green(),
            commit_count,
            goal_total_commits
        );
        io::stdout().flush().unwrap();
        spinner_index = (spinner_index + 1) % spinner_chars.len();
        thread::sleep(Duration::from_millis(100));
    }

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

    let mut remote = repo.find_remote("origin").expect("Could not find remote");
    let mut push_options = git2::PushOptions::new();
    push_options.remote_callbacks(callbacks);

    remote
        .push(&["refs/heads/main:refs/heads/main"], Some(&mut push_options))
        .expect("Could not push to remote");

    println!(
        "\n{}: Completed {} iterations and pushed to remote repository.",
        "Success".bold().green(),
        goal_total_commits - current_commit_count
    );
}
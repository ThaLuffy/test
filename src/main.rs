use git2::Repository;
use std::fs::{OpenOptions, File};
use std::io::Write;
use std::path::Path;

fn main() {
    let repo_path = Path::new(".");
    let repo = Repository::open(repo_path).expect("Could not open repository");

    let readme_path = repo_path.join("README.md");

    // Determine the current commit count
    let mut revwalk = repo.revwalk().expect("Could not get revwalk");
    revwalk.push_head().expect("Could not push head");
    let current_commit_count = revwalk.count();

    let mut commit_count = current_commit_count + 1;
    let goal_count = 100000; // Set your desired goal count here

    while commit_count < current_commit_count + goal_count {
        // Append 10 slashes to README.md and commit each addition
        for _ in 0..10 {
            if commit_count >= current_commit_count + goal_count {
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
                &format!("Add '/' character - iteration {} - total commits {}", commit_count, commit_count),
                &tree,
                &[&parent_commit],
            )
            .expect("Could not commit");

            commit_count += 1;
        }

        if commit_count >= current_commit_count + goal_count {
            break;
        }

        // Empty the contents of README.md and commit
        let mut readme_file = File::create(&readme_path).expect("Could not create README.md");
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
    }

    repo.find_remote("origin")
        .expect("Could not find remote")
        .push(&["refs/heads/main:refs/heads/main"], None)
        .expect("Could not push to remote");

    println!("Completed {} iterations and pushed to remote repository.", goal_count);
}
// src/services/git_service.rs

use git2::{Branch, BranchType, Commit, ObjectType, Oid, Repository, Signature, Status};
use std::path::Path;

#[derive(Debug)]
pub enum GitError {
    Git2(git2::Error),
    Io(std::io::Error),
    Other(String),
}

impl std::fmt::Display for GitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GitError::Git2(err) => write!(f, "{}", err),
            GitError::Io(err) => write!(f, "{}", err),
            GitError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl From<git2::Error> for GitError {
    fn from(err: git2::Error) -> Self {
        GitError::Git2(err)
    }
}

impl From<std::io::Error> for GitError {
    fn from(err: std::io::Error) -> Self {
        GitError::Io(err)
    }
}

pub struct GitService {}

impl GitService {
    pub fn new() -> Self {
        GitService {}
    }

    pub fn repository_exists(path: &Path) -> bool {
        Repository::open(path).is_ok()
    }

    // Initialize a new Git repository
    pub fn init_repository(path: &Path) -> Result<Repository, GitError> {
        Repository::init(path).map_err(GitError::from)
    }

    // Open an existing Git repository
    pub fn open_repository(path: &Path) -> Result<Repository, GitError> {
        Repository::open(path).map_err(GitError::from)
    }


    pub fn get_default_branch_name(repo: &Repository) -> Result<String, GitError> {
        let head = repo.head()?;
        let shorthand = head.shorthand();
        match shorthand {
            Some(branch_name) => Ok(branch_name.to_string()),
            None => Err(GitError::Other("Failed to get default branch name".to_string())),
        }
    }
    
    pub fn get_current_branch_name(repo: &Repository) -> Result<String, GitError> {
        let head = repo.head()?;
        let shorthand = head.shorthand();
        match shorthand {
            Some(branch_name) => Ok(branch_name.to_string()),
            None => Err(GitError::Other("Failed to get current branch name".to_string())),
        }
    }

    pub fn get_all_branch_names(repo: &Repository) -> Result<Vec<String>, GitError> {
        let mut branch_names = Vec::new();
        for branch_result in repo.branches(Some(BranchType::Local))? {
            let (branch, _) = branch_result?;
            if let Some(name) = branch.name()? {
                branch_names.push(name.to_string());
            }
        }
        Ok(branch_names)
    }

    pub fn create_branch<'repo>(
        repo: &'repo Repository,
        branch_name: &str,
        commit: &Commit<'repo>,
    ) -> Result<Branch<'repo>, GitError> {
        repo.branch(branch_name, commit, false).map_err(GitError::from)
    }

    pub fn checkout_branch(repo: &Repository, branch_name: &str) -> Result<(), GitError> {
        let branch = repo.find_branch(branch_name, BranchType::Local)?;
        let obj = branch.get().peel(ObjectType::Commit)?;
        repo.checkout_tree(&obj, None)?;
        repo.set_head(&format!("refs/heads/{}", branch_name))?;
        Ok(())
    }

    // Get the latest commit
    pub fn get_latest_commit<'repo>(repo: &'repo Repository) -> Result<Commit<'repo>, GitError> {
        let obj = repo.head()?.resolve()?.peel(ObjectType::Commit)?;
        repo.find_commit(obj.id()).map_err(GitError::from)
    }

    // Add a file to the staging area
    pub fn add_file_to_stage(repo: &Repository, file_path: &Path) -> Result<(), GitError> {
        let mut index = repo.index()?;
        index.add_path(file_path)?;
        index.write()?;
        Ok(())
    }

    // Commit changes
    pub fn commit_changes(
        repo: &Repository,
        author_name: &str, // Now parameters
        author_email: &str, // Now parameters
        message: &str,
    ) -> Result<Oid, GitError> {
        let mut index = repo.index()?;
        // Ensure all changes in the working directory are staged before creating the tree
        // This is a common pattern when committing everything.
        // For selective commits, `add_path` would be used per file.
        index.add_all(["."].iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;


        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;

        let head = repo.head()?;
        let commit = head.peel_to_commit()?;

        let signature = Signature::now(author_name, author_email)?;
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &[&commit],
        ).map_err(GitError::from)
    }

    pub fn delete_branch(repo: &Repository, branch_name: &str) -> Result<(), GitError> {
        let mut branch = repo.find_branch(branch_name, BranchType::Local)?;
        branch.delete()?;
        Ok(())
    }

    pub fn merge_branch(repo: &Repository, branch_name: &str) -> Result<(), GitError> {
        let branch = repo.find_branch(branch_name, BranchType::Local)?;
        let target = branch
            .get()
            .target()
            .ok_or(GitError::Other("Branch has no target".to_string()))?;
        let annotated_commit = repo.find_annotated_commit(target)?;

        let mut merge_options = git2::MergeOptions::new();
        merge_options.fail_on_conflict(true);

        repo.merge(&[&annotated_commit], Some(&mut merge_options), None)?;

        // Check if there are any conflicts
        let mut index = repo.index()?;
        if index.has_conflicts() {
            return Err(GitError::Other("Merge conflicts detected".to_string()));
        }

        // If the merge was successful, create a commit
        let signature = Signature::now("Your Name", "your.email@example.com")?; // Replace with actual user info
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        let head = repo.head()?;
        let commit = head.peel_to_commit()?;

        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            &format!("Merged branch '{}'", branch_name),
            &tree,
            &[&commit],
        )?;

        Ok(())
    }

    // New function to check for uncommitted changes
    pub fn has_uncommitted_changes(repo: &Repository) -> Result<bool, GitError> {
        let mut options = git2::StatusOptions::new();
        options.include_untracked(true); // Include untracked files
        options.recurse_untracked_dirs(true); // Recurse into untracked directories
        options.exclude_submodules(true); // Exclude submodules for simplicity

        let statuses = repo.statuses(Some(&mut options))?;

        // Iterate through statuses and check for any non-ignored, uncommitted changes
        for entry in statuses.iter() {
            let status = entry.status();
            // We're looking for any changes that are not 'ignored'
            if status != Status::empty() &&
               !status.is_ignored() &&
               !status.is_wt_new()
            {
                return Ok(true);
            }
        }
        Ok(false)
    }

    // Push changes to the remote repository
    pub fn push_to_remote(
        repo: &Repository,
        remote_name: &str,
        branch_name: &str,
    ) -> Result<(), GitError> {
        let mut remote = repo.find_remote(remote_name)?;

        // Setup credentials for push
        // This is a simplified approach. In a real-world application,
        // you might need more robust credential handling (e.g., SSH keys,
        // Git credential manager, explicit username/password/PAT).
        // git2 can often pick up credentials if they are configured
        // system-wide or via GIT_SSH_COMMAND/GIT_ASKPASS.
        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            // Try to get credentials from environment or configuration
            // For HTTPS, you might need a Personal Access Token (PAT)
            // For SSH, you might need an SSH key path
            eprintln!("Attempting to acquire credentials for user: {:?}", username_from_url);

            let username = username_from_url.unwrap_or("git");

            // Prioritize Personal Access Token (PAT) from environment variable for HTTPS
            // The user needs to set a GIT_PASSWORD env var with their PAT.
            if _allowed_types.is_user_pass_plaintext() {
                if let Ok(password) = std::env::var("GIT_PASSWORD") {
                    eprintln!("Using GIT_PASSWORD environment variable for user: {}", username);
                    return git2::Cred::userpass_plaintext(username, &password);
                }
            }

            // Fallback to SSH agent (for SSH remotes)
            if _allowed_types.is_ssh_key() {
                if let Ok(cred) = git2::Cred::ssh_key_from_agent(username) {
                    eprintln!("Using SSH agent for user: {}", username);
                    return Ok(cred);
                }
            }

            // Fallback to default credential lookup (e.g., .netrc, git credential manager)
            eprintln!("Falling back to default git credentials for user: {}", username);
            git2::Cred::default()
        });

        let mut options = git2::PushOptions::new();
        options.remote_callbacks(callbacks);

        let mut refspecs = Vec::new();
        refspecs.push(format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name));

        remote.push(&refspecs, Some(&mut options)).map_err(GitError::from)?;
        Ok(())
    }
}
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub repo_root: PathBuf,
    pub memory_dir: PathBuf,
}

impl Config {
    pub fn load() -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let repo_root = cwd.parent().unwrap_or(&cwd).to_path_buf();
        let memory_dir = repo_root.join("cowork");

        Self {
            repo_root,
            memory_dir,
        }
    }
}

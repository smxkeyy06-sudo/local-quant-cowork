use anyhow::Result;

use crate::prompts::{roles, system};
use crate::runtime::Runtime;

pub fn run(runtime: &Runtime) -> Result<()> {
    println!("cowork chat");
    println!("repo: {}", runtime.config.repo_root.display());
    println!("system: {}", system::SYSTEM_PROMPT_TITLE);
    println!("role: {}", roles::PRIMARY_ROLE);
    println!("status: starter chat loop not yet interactive");
    Ok(())
}

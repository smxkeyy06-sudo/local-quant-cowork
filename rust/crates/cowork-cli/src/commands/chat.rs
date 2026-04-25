use anyhow::Result;

use crate::prompts::{roles, system};
use crate::runtime::Runtime;

pub fn run(runtime: &Runtime) -> Result<()> {
    let _prompt_is_configured = !system::SYSTEM_PROMPT.trim().is_empty();
    let _role_profile_is_configured = !roles::QUANT_RESEARCH_ROLE.trim().is_empty();

    println!("cowork chat");
    println!("repo: {}", runtime.config.repo_root.display());
    println!("system: {}", system::SYSTEM_PROMPT_TITLE);
    println!("role: {}", roles::PRIMARY_ROLE);
    println!("status: starter chat loop not yet interactive");
    Ok(())
}

use anyhow::Result;

use crate::memory::files;
use crate::runtime::Runtime;

pub fn run(runtime: &Runtime) -> Result<()> {
    println!("cowork doctor");
    files::assert_required_files(&runtime.config.memory_dir)?;
    println!("memory: ok ({})", runtime.config.memory_dir.display());
    println!("tools: local registry stub enabled");
    Ok(())
}

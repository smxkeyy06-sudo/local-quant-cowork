use anyhow::Result;

use crate::memory::files;
use crate::runtime::Runtime;

pub fn run(runtime: &Runtime, goal: Option<&str>) -> Result<()> {
    let goal = goal.unwrap_or("(missing goal)");
    let tasks = files::read_tasks(&runtime.config.memory_dir)?;

    println!("cowork task");
    println!("goal: {goal}");
    println!("existing_tasks: {}", tasks.tasks.len());
    println!("note: append/update flow is next implementation step");
    Ok(())
}

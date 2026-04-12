#[derive(Debug, Clone)]
pub struct ToolDef {
    pub name: &'static str,
    pub description: &'static str,
}

pub fn default_tools() -> Vec<ToolDef> {
    vec![
        ToolDef { name: "read_file", description: "Read a file from the repository." },
        ToolDef { name: "write_file", description: "Write a file in the repository." },
        ToolDef { name: "edit_file", description: "Edit a file in place." },
        ToolDef { name: "glob_search", description: "List files via glob patterns." },
        ToolDef { name: "grep_search", description: "Search file content by pattern." },
        ToolDef { name: "bash", description: "Run local shell commands." },
    ]
}

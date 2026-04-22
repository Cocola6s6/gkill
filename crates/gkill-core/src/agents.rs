use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub id: &'static str,
    pub display_name: &'static str,
    pub skills_dir: &'static str,
    pub global_skills_dir: PathBuf,
}

pub fn all_agents() -> Vec<AgentConfig> {
    vec![
        make("claude-code", "Claude Code", ".claude/skills", claude_global()),
        make("codex", "Codex", ".agents/skills", codex_global()),
        make("cursor", "Cursor", ".agents/skills", simple_global(".cursor/skills")),
        make("antigravity", "Antigravity", ".agent/skills", simple_global(".gemini/antigravity/skills")),
        make("openclaw", "OpenClaw", "skills", openclaw_global()),
        make("opencode", "OpenCode", ".agents/skills", opencode_global()),
        make("trae", "Trae", ".trae/skills", simple_global(".trae/skills")),
        make("windsurf", "Windsurf", ".agents/skills", simple_global(".windsurf/skills")),
    ]
}

pub fn get_agent(name: &str) -> anyhow::Result<AgentConfig> {
    all_agents()
        .into_iter()
        .find(|a| a.id == name)
        .ok_or_else(|| anyhow::anyhow!("未知 Agent: {}。可选: {}", name, known_agents()))
}

pub fn known_agents() -> String {
    all_agents().iter().map(|a| a.id).collect::<Vec<_>>().join(", ")
}

fn home() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"))
}

fn make(id: &'static str, display_name: &'static str, skills_dir: &'static str, global: PathBuf) -> AgentConfig {
    AgentConfig { id, display_name, skills_dir, global_skills_dir: global }
}

fn simple_global(relative: &str) -> PathBuf {
    home().join(relative)
}

fn claude_global() -> PathBuf {
    if let Ok(dir) = std::env::var("CLAUDE_CONFIG_DIR") {
        let p = PathBuf::from(dir.trim());
        if !dir.trim().is_empty() { return p.join("skills"); }
    }
    home().join(".claude").join("skills")
}

fn codex_global() -> PathBuf {
    if let Ok(dir) = std::env::var("CODEX_HOME") {
        let p = PathBuf::from(dir.trim());
        if !dir.trim().is_empty() { return p.join("skills"); }
    }
    home().join(".codex").join("skills")
}

fn openclaw_global() -> PathBuf {
    for dir in &[".openclaw", ".clawdbot", ".moltbot"] {
        let p = home().join(dir).join("skills");
        if p.exists() { return p; }
    }
    home().join(".openclaw").join("skills")
}

fn opencode_global() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| home().join(".config"))
        .join("opencode")
        .join("skills")
}

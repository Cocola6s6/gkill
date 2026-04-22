use anyhow::Result;
use clap::Args;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use gkill_core::{api, config, http::Client, types::SkillItem};
use indicatif::{ProgressBar, ProgressStyle};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};
use std::io;

#[derive(Args)]
pub struct SearchArgs {
    /// 搜索关键词
    pub query: Option<String>,
    /// 排序方式 relevance|downloads
    #[arg(long, default_value = "relevance")]
    pub sort: String,
    /// 页码（从 0 开始）
    #[arg(long, default_value = "0")]
    pub page: u32,
    /// 每页数量
    #[arg(long, default_value = "12")]
    pub size: u32,
    /// SkillHub 地址
    #[arg(long)]
    pub registry: Option<String>,
}

pub async fn run(args: SearchArgs) -> Result<()> {
    let token = config::read_token()?;
    let registry = super::registry(args.registry);
    let client = Client::new(&registry, token);
    let query = args.query.unwrap_or_default();

    if !std::io::IsTerminal::is_terminal(&std::io::stdout()) {
        // Non-TTY: plain text output
        return run_plain(&client, &query, &args.sort, args.page, args.size).await;
    }

    run_tui(client, query, args.sort, args.size).await
}

async fn run_plain(client: &Client, q: &str, sort: &str, page: u32, size: u32) -> Result<()> {
    let pb = spinner("正在搜索...");
    let result = api::search_skills(client, q, sort, page, size).await?;
    pb.finish_and_clear();
    println!("共 {} 个结果：", result.total);
    for item in &result.items {
        println!(
            "  {} ({}) - {}",
            item.slug,
            item.namespace.as_deref().unwrap_or("global"),
            item.summary.as_deref().unwrap_or("")
        );
    }
    Ok(())
}

struct TuiState {
    client: Client,
    query: String,
    sort: String,
    size: u32,
    page: u32,
    total_pages: u32,
    items: Vec<SkillItem>,
    selected: usize,
    loading: bool,
    status_msg: String,
}

impl TuiState {
    fn new(client: Client, query: String, sort: String, size: u32) -> Self {
        Self {
            client,
            query,
            sort,
            size,
            page: 0,
            total_pages: 1,
            items: vec![],
            selected: 0,
            loading: true,
            status_msg: String::new(),
        }
    }

    async fn load(&mut self) -> Result<()> {
        self.loading = true;
        let result = api::search_skills(&self.client, &self.query, &self.sort, self.page, self.size).await?;
        self.total_pages = ((result.total + self.size as u64 - 1) / self.size as u64).max(1) as u32;
        self.items = result.items;
        self.selected = 0;
        self.loading = false;
        Ok(())
    }
}

async fn run_tui(client: Client, query: String, sort: String, size: u32) -> Result<()> {
    let mut state = TuiState::new(client, query, sort, size);

    // Initial load before entering TUI
    let pb = spinner("正在搜索...");
    state.load().await?;
    pb.finish_and_clear();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = tui_loop(&mut terminal, &mut state).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn tui_loop(
    terminal: &mut Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
    state: &mut TuiState,
) -> Result<()> {
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // header
                    Constraint::Min(0),    // list
                    Constraint::Length(2), // footer
                ])
                .split(f.area());

            // Header
            let header = Paragraph::new(format!(
                " 搜索: {}    第 {}/{} 页  共 {} 个结果",
                if state.query.is_empty() { "（全部）".to_string() } else { state.query.clone() },
                state.page + 1,
                state.total_pages,
                state.items.len()
            ))
            .block(Block::default().title(" gkill search ").borders(Borders::ALL))
            .style(Style::default().fg(Color::Cyan));
            f.render_widget(header, chunks[0]);

            // Skill list
            let items: Vec<ListItem> = state
                .items
                .iter()
                .enumerate()
                .map(|(i, s)| {
                    let ns = s.namespace.as_deref().unwrap_or("global");
                    let desc = s.summary.as_deref().unwrap_or("");
                    let downloads = s.download_count.unwrap_or(0);
                    let line = Line::from(vec![
                        Span::styled(
                            format!("  {:<30}", s.slug),
                            if i == state.selected {
                                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                            } else {
                                Style::default()
                            },
                        ),
                        Span::styled(format!("{:<12}", ns), Style::default().fg(Color::Green)),
                        Span::styled(format!(" ↓{:<8}", downloads), Style::default().fg(Color::DarkGray)),
                        Span::raw(desc.chars().take(40).collect::<String>()),
                    ]);
                    ListItem::new(line)
                })
                .collect();

            let mut list_state = ListState::default();
            list_state.select(Some(state.selected));
            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL))
                .highlight_style(Style::default().bg(Color::DarkGray));
            f.render_stateful_widget(list, chunks[1], &mut list_state);

            // Footer
            let footer_text = if !state.status_msg.is_empty() {
                state.status_msg.clone()
            } else {
                " ↑↓ 导航  ← → 翻页  Enter 安装  q 退出".to_string()
            };
            let footer = Paragraph::new(footer_text)
                .style(Style::default().fg(Color::DarkGray));
            f.render_widget(footer, chunks[2]);
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press { continue; }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Up => {
                        if state.selected > 0 { state.selected -= 1; }
                    }
                    KeyCode::Down => {
                        if state.selected + 1 < state.items.len() { state.selected += 1; }
                    }
                    KeyCode::Left => {
                        if state.page > 0 {
                            state.page -= 1;
                            let _ = state.load().await;
                        }
                    }
                    KeyCode::Right => {
                        if state.page + 1 < state.total_pages {
                            state.page += 1;
                            let _ = state.load().await;
                        }
                    }
                    KeyCode::Enter => {
                        if let Some(skill) = state.items.get(state.selected) {
                            state.status_msg = format!("正在安装 {}...", skill.slug);
                            // Exit TUI and install
                            let slug = skill.slug.clone();
                            let ns = skill.namespace.clone().unwrap_or_else(|| "global".to_string());
                            // We'll return the selection via a signal
                            // For simplicity: exit TUI then install
                            let _ = &terminal; // suppress drop-ref warning
                            // Re-setup terminal handled by caller's cleanup
                            return install_after_tui(&state.client, &slug, &ns).await;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

async fn install_after_tui(client: &Client, slug: &str, ns: &str) -> Result<()> {
    // Restore terminal state is handled by the caller
    use gkill_core::agents;
    use gkill_core::install;
    use dialoguer::Select;

    let all = agents::all_agents();
    let names: Vec<String> = all.iter().map(|a| format!("{} ({})", a.display_name, a.id)).collect();
    let idx = Select::new()
        .with_prompt("选择 Agent")
        .items(&names)
        .default(0)
        .interact()?;
    let agent = all[idx].clone();

    let modes = ["global - 全局安装", "project - 项目安装"];
    let midx = Select::new()
        .with_prompt("选择安装模式")
        .items(&modes)
        .default(0)
        .interact()?;
    let mode = if midx == 0 { "global" } else { "project" };

    let pb = spinner(&format!("正在安装 {}...", slug));
    install::install_skill(client, slug, ns, &agent, mode, None).await?;
    pb.finish_and_clear();
    let dir = install::skill_dir(&agent, mode, slug);
    println!("✅ 已安装 {} → {}", slug, dir.display());
    Ok(())
}

fn spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner} {msg}").unwrap());
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    pb
}

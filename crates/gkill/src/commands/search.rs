use anyhow::Result;
use clap::Args;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use gkill_core::{agents, api, config, http::Client, types::SkillItem};
use indicatif::{ProgressBar, ProgressStyle};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};
use std::collections::BTreeSet;
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
    selected_indices: BTreeSet<usize>,
    phase: TuiPhase,
    agent_cursor: usize,
    selected_agent_indices: BTreeSet<usize>,
    pending_skills: Vec<(String, String)>,
    agents: Vec<agents::AgentConfig>,
    loading: bool,
    status_msg: String,
}

enum TuiPhase {
    SkillSelect,
    AgentSelect,
}

enum TuiAction {
    Quit,
    InstallMany {
        skills: Vec<(String, String)>,
        agents: Vec<agents::AgentConfig>,
    },
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
            selected_indices: BTreeSet::new(),
            phase: TuiPhase::SkillSelect,
            agent_cursor: 0,
            selected_agent_indices: BTreeSet::new(),
            pending_skills: vec![],
            agents: agents::all_agents(),
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
        self.selected_indices.clear();
        self.loading = false;
        Ok(())
    }

    fn agent_option_count(&self) -> usize {
        self.agents.len() + 1
    }

    fn toggle_agent_current(&mut self) {
        if self.agent_option_count() == 0 {
            return;
        }
        let idx = self.agent_cursor;
        if idx == 0 {
            if self.selected_agent_indices.contains(&0) {
                self.selected_agent_indices.clear();
            } else {
                self.selected_agent_indices = (0..self.agent_option_count()).collect();
            }
            return;
        }
        if !self.selected_agent_indices.insert(idx) {
            self.selected_agent_indices.remove(&idx);
        }
        self.sync_all_agent_marker();
    }

    fn toggle_all_agents(&mut self) {
        if self.selected_real_agent_count() == self.agents.len() {
            self.selected_agent_indices.clear();
            return;
        }
        self.selected_agent_indices = (0..self.agent_option_count()).collect();
    }

    fn sync_all_agent_marker(&mut self) {
        if self.agents.is_empty() {
            self.selected_agent_indices.remove(&0);
            return;
        }
        let real_selected = self.selected_agent_indices.iter().filter(|i| **i > 0).count();
        if real_selected == self.agents.len() {
            self.selected_agent_indices.insert(0);
        } else {
            self.selected_agent_indices.remove(&0);
        }
    }

    fn selected_real_agent_count(&self) -> usize {
        if self.selected_agent_indices.contains(&0) {
            return self.agents.len();
        }
        self.selected_agent_indices.iter().filter(|i| **i > 0).count()
    }

    fn selected_agents(&self) -> Vec<agents::AgentConfig> {
        if self.selected_agent_indices.contains(&0) {
            return self.agents.clone();
        }
        self.selected_agent_indices
            .iter()
            .filter(|idx| **idx > 0)
            .filter_map(|idx| self.agents.get(*idx - 1))
            .cloned()
            .collect()
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

    let action = tui_loop(&mut terminal, &mut state).await?;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    match action {
        TuiAction::Quit => Ok(()),
        TuiAction::InstallMany { skills, agents } => {
            install_after_tui(&state.client, &skills, &agents).await
        }
    }
}

async fn tui_loop(
    terminal: &mut Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
    state: &mut TuiState,
) -> Result<TuiAction> {
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

            match state.phase {
                TuiPhase::SkillSelect => {
                    let header = Paragraph::new(format!(
                        " 搜索: {}    第 {}/{} 页  本页 {} 个结果",
                        if state.query.is_empty() { "（全部）".to_string() } else { state.query.clone() },
                        state.page + 1,
                        state.total_pages,
                        state.items.len()
                    ))
                    .block(Block::default().title(" gkill search ").borders(Borders::ALL))
                    .style(Style::default().fg(Color::Cyan));
                    f.render_widget(header, chunks[0]);

                    let items: Vec<ListItem> = state
                        .items
                        .iter()
                        .enumerate()
                        .map(|(i, s)| {
                            let ns = s.namespace.as_deref().unwrap_or("global");
                            let desc = s.summary.as_deref().unwrap_or("");
                            let downloads = s.download_count.unwrap_or(0);
                            let checked = state.selected_indices.contains(&i);
                            let line = Line::from(vec![
                                Span::styled(
                                    format!("[{}] ", if checked { "x" } else { " " }),
                                    if checked {
                                        Style::default().fg(Color::LightGreen)
                                    } else {
                                        Style::default().fg(Color::DarkGray)
                                    },
                                ),
                                Span::styled(
                                    format!("{:<30}", s.slug),
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

                    let footer_text = if !state.status_msg.is_empty() {
                        state.status_msg.clone()
                    } else {
                        format!(
                            " ↑↓ 导航  ← → 翻页  Space 选中  A 全选本页  Enter 下一步(选Agent)  q 退出  已选:{}",
                            state.selected_indices.len()
                        )
                    };
                    let footer = Paragraph::new(footer_text).style(Style::default().fg(Color::DarkGray));
                    f.render_widget(footer, chunks[2]);
                }
                TuiPhase::AgentSelect => {
                    let header = Paragraph::new(format!(
                        " 待安装 Skill: {} 个    选择安装 Agent",
                        state.pending_skills.len()
                    ))
                    .block(Block::default().title(" gkill install ").borders(Borders::ALL))
                    .style(Style::default().fg(Color::Cyan));
                    f.render_widget(header, chunks[0]);

                    let mut rows = Vec::with_capacity(state.agent_option_count());
                    let all_checked = state.selected_agent_indices.contains(&0);
                    rows.push(ListItem::new(Line::from(vec![
                        Span::styled(
                            format!("[{}] ", if all_checked { "x" } else { " " }),
                            if all_checked {
                                Style::default().fg(Color::LightGreen)
                            } else {
                                Style::default().fg(Color::DarkGray)
                            },
                        ),
                        Span::styled("全部 Agent", Style::default().fg(Color::Yellow)),
                    ])));

                    for (idx, a) in state.agents.iter().enumerate() {
                        let checked = state.selected_agent_indices.contains(&(idx + 1));
                        rows.push(ListItem::new(Line::from(vec![
                            Span::styled(
                                format!("[{}] ", if checked { "x" } else { " " }),
                                if checked {
                                    Style::default().fg(Color::LightGreen)
                                } else {
                                    Style::default().fg(Color::DarkGray)
                                },
                            ),
                            Span::raw(format!("{} ({})", a.display_name, a.id)),
                        ])));
                    }

                    let mut list_state = ListState::default();
                    list_state.select(Some(state.agent_cursor));
                    let list = List::new(rows)
                        .block(Block::default().borders(Borders::ALL))
                        .highlight_style(Style::default().bg(Color::DarkGray));
                    f.render_stateful_widget(list, chunks[1], &mut list_state);

                    let footer_text = if !state.status_msg.is_empty() {
                        state.status_msg.clone()
                    } else {
                        format!(
                            " ↑↓ 导航  Space 选中  A 全选  Enter 开始安装  Esc 返回  q 退出  已选:{}",
                            state.selected_real_agent_count()
                        )
                    };
                    let footer = Paragraph::new(footer_text).style(Style::default().fg(Color::DarkGray));
                    f.render_widget(footer, chunks[2]);
                }
            }
        })?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press { continue; }
                match state.phase {
                    TuiPhase::SkillSelect => match key.code {
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
                        KeyCode::Char(' ') => {
                            if !state.items.is_empty() {
                                if !state.selected_indices.insert(state.selected) {
                                    state.selected_indices.remove(&state.selected);
                                }
                            }
                        }
                        KeyCode::Char('a') | KeyCode::Char('A') => {
                            if !state.items.is_empty() {
                                if state.selected_indices.len() == state.items.len() {
                                    state.selected_indices.clear();
                                } else {
                                    state.selected_indices = (0..state.items.len()).collect();
                                }
                            }
                        }
                        KeyCode::Enter => {
                            if let Some(skill) = state.items.get(state.selected) {
                                state.pending_skills = if state.selected_indices.is_empty() {
                                    vec![(
                                        skill.slug.clone(),
                                        skill.namespace.clone().unwrap_or_else(|| "global".to_string()),
                                    )]
                                } else {
                                    state
                                        .selected_indices
                                        .iter()
                                        .filter_map(|idx| state.items.get(*idx))
                                        .map(|s| {
                                            (
                                                s.slug.clone(),
                                                s.namespace.clone().unwrap_or_else(|| "global".to_string()),
                                            )
                                        })
                                        .collect()
                                };
                                state.phase = TuiPhase::AgentSelect;
                                state.agent_cursor = 0;
                                state.selected_agent_indices.clear();
                                state.status_msg.clear();
                            }
                        }
                        _ => {}
                    },
                    TuiPhase::AgentSelect => match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Esc => {
                            state.phase = TuiPhase::SkillSelect;
                            state.status_msg.clear();
                        }
                        KeyCode::Up => {
                            if state.agent_cursor > 0 {
                                state.agent_cursor -= 1;
                            }
                        }
                        KeyCode::Down => {
                            if state.agent_cursor + 1 < state.agent_option_count() {
                                state.agent_cursor += 1;
                            }
                        }
                        KeyCode::Char(' ') => state.toggle_agent_current(),
                        KeyCode::Char('a') | KeyCode::Char('A') => state.toggle_all_agents(),
                        KeyCode::Enter => {
                            let selected_agents = state.selected_agents();
                            if selected_agents.is_empty() {
                                state.status_msg = "请至少选择一个 Agent（Space 或 A）".to_string();
                            } else {
                                return Ok(TuiAction::InstallMany {
                                    skills: state.pending_skills.clone(),
                                    agents: selected_agents,
                                });
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    Ok(TuiAction::Quit)
}

async fn install_after_tui(
    client: &Client,
    skills: &[(String, String)],
    selected_agents: &[agents::AgentConfig],
) -> Result<()> {
    // Restore terminal state is handled by the caller
    use gkill_core::install;
    use dialoguer::Select;

    let modes = ["global - 全局安装", "project - 项目安装"];
    let midx = Select::new()
        .with_prompt("选择安装模式")
        .items(&modes)
        .default(0)
        .interact()?;
    let mode = if midx == 0 { "global" } else { "project" };

    for agent in selected_agents {
        for (slug, ns) in skills {
            let pb = spinner(&format!("正在安装 {} 到 {}...", slug, agent.id));
            install::install_skill(client, slug, ns, agent, mode, None).await?;
            pb.finish_and_clear();
            let dir = install::skill_dir(agent, mode, slug);
            println!("✅ [{}] 已安装 {} → {}", agent.id, slug, dir.display());
        }
    }
    let total = skills.len() * selected_agents.len();
    if total > 1 {
        println!(
            "批量安装完成，共 {} 项（{} skills x {} agents）",
            total,
            skills.len(),
            selected_agents.len()
        );
    }
    Ok(())
}

fn spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner} {msg}").unwrap());
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    pb
}

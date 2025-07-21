use crate::agent::{self, Agent};
use crate::store::{self, Board, Okr, Task, TaskStatus};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use notify::{recommended_watcher, RecursiveMode, Watcher};
use ratatui::{
    prelude::*,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};
use std::io;
use std::path::Path;
use std::sync::{mpsc::channel, Arc, Mutex};
use std::time::Duration;

enum View {
    Board,
    TaskDescription,
    AssignAgent,
    AddComment,
    AddTask,
    UpdateTask,
    Logs,
    Agents,
    Okrs,
    Commands,
}

struct App {
    board: Arc<Mutex<Board>>,
    agents: Vec<Agent>,
    selected_column: usize,
    selected_task: [ListState; 3],
    current_view: View,
    agent_list_state: ListState,
    comment_input: String,
    new_task_title: String,
    new_task_description: String,
    editing_description: bool,
    logs: String,
    okrs: Vec<Okr>,
}

impl App {
    fn new(board: Board, agents: Vec<Agent>) -> Self {
        let mut app = App {
            board: Arc::new(Mutex::new(board)),
            agents,
            selected_column: 0,
            selected_task: [
                ListState::default(),
                ListState::default(),
                ListState::default(),
            ],
            current_view: View::Board,
            agent_list_state: ListState::default(),
            comment_input: String::new(),
            new_task_title: String::new(),
            new_task_description: String::new(),
            editing_description: false,
            logs: std::fs::read_to_string(".taskter/logs.log").unwrap_or_default(),
            okrs: store::load_okrs().unwrap_or_default(),
        };
        app.selected_task[0].select(Some(0));
        app
    }

    fn next_column(&mut self) {
        self.selected_column = (self.selected_column + 1) % 3;
        self.ensure_selected_task();
    }

    fn prev_column(&mut self) {
        self.selected_column = (self.selected_column + 2) % 3;
        self.ensure_selected_task();
    }

    fn ensure_selected_task(&mut self) {
        let tasks = self.tasks_in_current_column();
        if !tasks.is_empty()
            && self.selected_task[self.selected_column]
                .selected()
                .is_none()
        {
            self.selected_task[self.selected_column].select(Some(0));
        }
    }

    fn next_task(&mut self) {
        let tasks = self.tasks_in_current_column();
        if tasks.is_empty() {
            return;
        }
        let i = match self.selected_task[self.selected_column].selected() {
            Some(i) => (i + 1) % tasks.len(),
            None => 0,
        };
        self.selected_task[self.selected_column].select(Some(i));
    }

    fn prev_task(&mut self) {
        let tasks = self.tasks_in_current_column();
        if tasks.is_empty() {
            return;
        }
        let i = match self.selected_task[self.selected_column].selected() {
            Some(i) => (i + tasks.len() - 1) % tasks.len(),
            None => 0,
        };
        self.selected_task[self.selected_column].select(Some(i));
    }

    fn tasks_in_current_column(&self) -> Vec<Task> {
        let status = match self.selected_column {
            0 => TaskStatus::ToDo,
            1 => TaskStatus::InProgress,
            _ => TaskStatus::Done,
        };
        self.board
            .lock()
            .unwrap()
            .tasks
            .iter()
            .filter(|t| t.status == status)
            .cloned()
            .collect()
    }

    fn move_task_to_next_column(&mut self) {
        self.move_task(1);
    }

    fn move_task_to_prev_column(&mut self) {
        self.move_task(-1);
    }

    fn move_task(&mut self, direction: i8) {
        let task_id_to_move =
            if let Some(selected_index) = self.selected_task[self.selected_column].selected() {
                let tasks_in_column = self.tasks_in_current_column();
                tasks_in_column.get(selected_index).map(|t| t.id)
            } else {
                None
            };

        if let Some(task_id) = task_id_to_move {
            if let Some(task) = self
                .board
                .lock()
                .unwrap()
                .tasks
                .iter_mut()
                .find(|t| t.id == task_id)
            {
                let current_status_index = task.status.clone() as usize;
                let next_status_index = (current_status_index as i8 + direction + 3) % 3;
                task.status = match next_status_index {
                    0 => TaskStatus::ToDo,
                    1 => TaskStatus::InProgress,
                    _ => TaskStatus::Done,
                };
            }
        }
    }

    fn get_selected_task(&self) -> Option<Task> {
        self.selected_task[self.selected_column]
            .selected()
            .and_then(|selected_index| {
                let tasks_in_column = self.tasks_in_current_column();
                tasks_in_column.get(selected_index).cloned()
            })
    }
}

pub fn run_tui() -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let board = store::load_board().unwrap_or_default();
    let agents = agent::load_agents().unwrap_or_default();
    let app = App::new(board, agents);
    let res = run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}")
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    // channel receiving filesystem events
    let (tx, rx) = channel();
    // set up watcher to monitor board, okrs, logs and agents files
    let mut watcher = recommended_watcher(move |res| {
        let _ = tx.send(res);
    })
    .map_err(io::Error::other)?;

    for path in [
        ".taskter/board.json",
        ".taskter/okrs.json",
        ".taskter/logs.log",
        ".taskter/agents.json",
    ] {
        watcher
            .watch(Path::new(path), RecursiveMode::NonRecursive)
            .map_err(io::Error::other)?;
    }

    loop {
        // handle filesystem events and reload data if needed
        while let Ok(res) = rx.try_recv() {
            if let Ok(event) = res {
                for p in event.paths {
                    if p.ends_with("board.json") {
                        if let Ok(board) = store::load_board() {
                            *app.board.lock().unwrap() = board;
                        }
                    } else if p.ends_with("okrs.json") {
                        if let Ok(okrs) = store::load_okrs() {
                            app.okrs = okrs;
                        }
                    } else if p.ends_with("logs.log") {
                        if let Ok(logs) = std::fs::read_to_string(".taskter/logs.log") {
                            app.logs = logs;
                        }
                    } else if p.ends_with("agents.json") {
                        if let Ok(agents) = crate::agent::load_agents() {
                            app.agents = agents;
                        }
                    }
                }
            }
        }

        terminal.draw(|f| ui(f, &mut app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match app.current_view {
                    View::Board => match key.code {
                        KeyCode::Char('q') => {
                            store::save_board(&app.board.lock().unwrap()).unwrap();
                            return Ok(());
                        }
                        KeyCode::Right | KeyCode::Tab => app.next_column(),
                        KeyCode::Left => app.prev_column(),
                        KeyCode::Down => app.next_task(),
                        KeyCode::Up => app.prev_task(),
                        KeyCode::Char('l') => app.move_task_to_next_column(),
                        KeyCode::Char('h') => app.move_task_to_prev_column(),
                        KeyCode::Enter => {
                            if app.get_selected_task().is_some() {
                                app.current_view = View::TaskDescription;
                            }
                        }
                        KeyCode::Char('a') => {
                            if app.get_selected_task().is_some() {
                                app.current_view = View::AssignAgent;
                                app.agent_list_state.select(Some(0));
                            }
                        }
                        KeyCode::Char('c') => {
                            if app.get_selected_task().is_some() {
                                app.current_view = View::AddComment;
                                app.comment_input.clear();
                            }
                        }
                        KeyCode::Char('n') => {
                            app.new_task_title.clear();
                            app.new_task_description.clear();
                            app.editing_description = false;
                            app.current_view = View::AddTask;
                        }
                        KeyCode::Char('u') => {
                            if let Some(task) = app.get_selected_task() {
                                app.new_task_title = task.title;
                                app.new_task_description = task.description.unwrap_or_default();
                                app.editing_description = false;
                                app.current_view = View::UpdateTask;
                            }
                        }
                        KeyCode::Char('d') => {
                            if let Some(task_id) = app.get_selected_task().map(|t| t.id) {
                                app.board.lock().unwrap().remove_task(task_id);
                                let tasks = app.tasks_in_current_column();
                                if !tasks.is_empty() {
                                    app.selected_task[app.selected_column].select(Some(0));
                                } else {
                                    app.selected_task[app.selected_column].select(None);
                                }
                                store::save_board(&app.board.lock().unwrap()).unwrap();
                            }
                        }
                        KeyCode::Char('L') => {
                            app.logs =
                                std::fs::read_to_string(".taskter/logs.log").unwrap_or_default();
                            app.current_view = View::Logs;
                        }
                        KeyCode::Char('A') => {
                            app.agents = crate::agent::load_agents().unwrap_or_default();
                            app.current_view = View::Agents;
                        }
                        KeyCode::Char('O') => {
                            app.okrs = store::load_okrs().unwrap_or_default();
                            app.current_view = View::Okrs;
                        }
                        KeyCode::Char('?') => {
                            app.current_view = View::Commands;
                        }
                        _ => {}
                    },
                    View::TaskDescription => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            app.current_view = View::Board;
                        }
                        _ => {}
                    },
                    View::AssignAgent => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            app.current_view = View::Board;
                        }
                        KeyCode::Down => {
                            let i = match app.agent_list_state.selected() {
                                Some(i) => (i + 1) % app.agents.len(),
                                None => 0,
                            };
                            app.agent_list_state.select(Some(i));
                        }
                        KeyCode::Up => {
                            let i = match app.agent_list_state.selected() {
                                Some(i) => (i + app.agents.len() - 1) % app.agents.len(),
                                None => 0,
                            };
                            app.agent_list_state.select(Some(i));
                        }
                        KeyCode::Enter => {
                            if let Some(selected_agent_index) = app.agent_list_state.selected() {
                                if let Some(agent) = app.agents.get(selected_agent_index).cloned() {
                                    if let Some(task) = app.get_selected_task() {
                                        let mut board = app.board.lock().unwrap();
                                        board.assign_agent(task.id, agent.id);
                                        let agent_clone = agent.clone();
                                        let task_clone = task.clone();
                                        let board_clone = Arc::clone(&app.board);
                                        tokio::spawn(async move {
                                            let result =
                                                agent::execute_task(&agent_clone, &task_clone)
                                                    .await;
                                            let mut board = board_clone.lock().unwrap();
                                            if let Some(task) = board
                                                .tasks
                                                .iter_mut()
                                                .find(|t| t.id == task_clone.id)
                                            {
                                                match result {
                                                    Ok(result) => match result {
                                                        agent::ExecutionResult::Success {
                                                            comment,
                                                        } => {
                                                            task.status = store::TaskStatus::Done;
                                                            task.comment = Some(comment);
                                                        }
                                                        agent::ExecutionResult::Failure {
                                                            comment,
                                                        } => {
                                                            task.status = store::TaskStatus::ToDo;
                                                            task.comment = Some(comment);
                                                            task.agent_id = None;
                                                        }
                                                    },
                                                    Err(_) => {
                                                        task.status = store::TaskStatus::ToDo;
                                                        task.comment = Some(
                                                            "Failed to execute task.".to_string(),
                                                        );
                                                        task.agent_id = None;
                                                    }
                                                }
                                            }
                                            store::save_board(&board).unwrap();
                                        });
                                    }
                                }
                            }
                            app.current_view = View::Board;
                        }
                        _ => {}
                    },
                    View::AddComment => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            app.current_view = View::Board;
                        }
                        KeyCode::Enter => {
                            if let Some(task_id) = app.get_selected_task().map(|t| t.id) {
                                let mut board = app.board.lock().unwrap();
                                board.add_comment(task_id, app.comment_input.clone());
                                store::save_board(&board).unwrap();
                            }
                            app.current_view = View::Board;
                        }
                        KeyCode::Backspace => {
                            app.comment_input.pop();
                        }
                        KeyCode::Char(c) => {
                            app.comment_input.push(c);
                        }
                        _ => {}
                    },
                    View::AddTask => match key.code {
                        KeyCode::Char(c) => {
                            if app.editing_description {
                                app.new_task_description.push(c);
                            } else {
                                app.new_task_title.push(c);
                            }
                        }
                        KeyCode::Backspace => {
                            if app.editing_description {
                                app.new_task_description.pop();
                            } else {
                                app.new_task_title.pop();
                            }
                        }
                        KeyCode::Enter => {
                            if app.editing_description {
                                let mut board = app.board.lock().unwrap();
                                board.add_task(
                                    app.new_task_title.clone(),
                                    if app.new_task_description.is_empty() {
                                        None
                                    } else {
                                        Some(app.new_task_description.clone())
                                    },
                                );
                                store::save_board(&board).unwrap();
                                app.current_view = View::Board;
                                app.editing_description = false;
                            } else {
                                app.editing_description = true;
                            }
                        }
                        KeyCode::Esc => {
                            app.current_view = View::Board;
                            app.editing_description = false;
                        }
                        _ => {}
                    },
                    View::UpdateTask => match key.code {
                        KeyCode::Char(c) => {
                            if app.editing_description {
                                app.new_task_description.push(c);
                            } else {
                                app.new_task_title.push(c);
                            }
                        }
                        KeyCode::Backspace => {
                            if app.editing_description {
                                app.new_task_description.pop();
                            } else {
                                app.new_task_title.pop();
                            }
                        }
                        KeyCode::Enter => {
                            if app.editing_description {
                                if let Some(task_id) = app.get_selected_task().map(|t| t.id) {
                                    let mut board = app.board.lock().unwrap();
                                    board.update_task(
                                        task_id,
                                        app.new_task_title.clone(),
                                        if app.new_task_description.is_empty() {
                                            None
                                        } else {
                                            Some(app.new_task_description.clone())
                                        },
                                    );
                                    store::save_board(&board).unwrap();
                                }
                                app.current_view = View::Board;
                                app.editing_description = false;
                            } else {
                                app.editing_description = true;
                            }
                        }
                        KeyCode::Esc => {
                            app.current_view = View::Board;
                            app.editing_description = false;
                        }
                        _ => {}
                    },
                    View::Logs | View::Agents | View::Okrs | View::Commands => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('?') => {
                            app.current_view = View::Board;
                        }
                        _ => {}
                    },
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    render_board(f, app);
    match app.current_view {
        View::TaskDescription => render_task_description(f, app),
        View::AssignAgent => render_assign_agent(f, app),
        View::AddComment => render_add_comment(f, app),
        View::AddTask => render_add_task(f, app),
        View::UpdateTask => render_update_task(f, app),
        View::Logs => render_logs(f, app),
        View::Agents => render_agents_list(f, app),
        View::Okrs => render_okrs(f, app),
        View::Commands => render_commands(f),
        _ => {}
    }
}

fn render_board(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(34),
            ]
            .as_ref(),
        )
        .split(f.area());

    for (i, status) in [TaskStatus::ToDo, TaskStatus::InProgress, TaskStatus::Done]
        .iter()
        .enumerate()
    {
        let tasks: Vec<ListItem> = app
            .board
            .lock()
            .unwrap()
            .tasks
            .iter()
            .filter(|t| t.status == *status)
            .map(|t| {
                let title = if t.agent_id.is_some() {
                    format!("* {}", t.title)
                } else {
                    t.title.clone()
                };
                ListItem::new(title)
            })
            .collect();
        let mut list = List::new(tasks).block(
            Block::default()
                .title(format!("{status:?}"))
                .borders(Borders::ALL),
        );
        if app.selected_column == i {
            list = list.highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Blue),
            );
        }
        f.render_stateful_widget(list, chunks[i], &mut app.selected_task[i]);
    }
}

fn render_task_description(f: &mut Frame, app: &mut App) {
    if let Some(task) = app.get_selected_task() {
        let mut text = vec![
            Line::from(Span::styled(
                task.title.clone(),
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(task.description.clone().unwrap_or_default()),
        ];

        if let Some(agent_id) = task.agent_id {
            text.push(Line::from(format!("Assigned to agent: {agent_id}")));
        }

        if let Some(comment) = &task.comment {
            text.push(Line::from(Span::styled(
                format!("Comment: {comment}"),
                Style::default().fg(Color::Yellow),
            )));
        }

        let block = Block::default()
            .title("Task Description")
            .borders(Borders::ALL);
        let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
        let area = centered_rect(60, 25, f.area());
        f.render_widget(Clear, area); //this clears the background
        f.render_widget(paragraph, area);
    }
}

fn render_assign_agent(f: &mut Frame, app: &mut App) {
    if app.agents.is_empty() {
        let block = Block::default().title("Assign Agent").borders(Borders::ALL);
        let text = Paragraph::new("No agents available. Create one with `taskter add-agent`")
            .block(block)
            .wrap(Wrap { trim: true });
        let area = centered_rect(60, 25, f.area());
        f.render_widget(Clear, area);
        f.render_widget(text, area);
        return;
    }

    let agents: Vec<ListItem> = app
        .agents
        .iter()
        .map(|a| ListItem::new(a.system_prompt.as_str()))
        .collect();

    let agent_list = List::new(agents)
        .block(Block::default().title("Assign Agent").borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Blue),
        );

    let area = centered_rect(60, 25, f.area());
    f.render_widget(Clear, area);
    f.render_stateful_widget(agent_list, area, &mut app.agent_list_state);
}

fn render_add_comment(f: &mut Frame, app: &mut App) {
    let block = Block::default().title("Add Comment").borders(Borders::ALL);
    let paragraph = Paragraph::new(app.comment_input.as_str())
        .block(block)
        .wrap(Wrap { trim: true });
    let area = centered_rect(60, 25, f.area());
    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
}

fn render_add_task(f: &mut Frame, app: &mut App) {
    let block = Block::default().title("New Task").borders(Borders::ALL);
    let title_style = if !app.editing_description {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let desc_style = if app.editing_description {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let paragraph = Paragraph::new(vec![
        Line::from(vec![
            Span::raw("Title: "),
            Span::styled(app.new_task_title.as_str(), title_style),
        ]),
        Line::from(vec![
            Span::raw("Description: "),
            Span::styled(app.new_task_description.as_str(), desc_style),
        ]),
    ])
    .block(block)
    .wrap(Wrap { trim: true });
    let area = centered_rect(60, 15, f.area());
    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
}

fn render_update_task(f: &mut Frame, app: &mut App) {
    let block = Block::default().title("Edit Task").borders(Borders::ALL);
    let title_style = if !app.editing_description {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let desc_style = if app.editing_description {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };
    let paragraph = Paragraph::new(vec![
        Line::from(vec![
            Span::raw("Title: "),
            Span::styled(app.new_task_title.as_str(), title_style),
        ]),
        Line::from(vec![
            Span::raw("Description: "),
            Span::styled(app.new_task_description.as_str(), desc_style),
        ]),
    ])
    .block(block)
    .wrap(Wrap { trim: true });
    let area = centered_rect(60, 15, f.area());
    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
}

fn render_logs(f: &mut Frame, app: &mut App) {
    let block = Block::default().title("Logs").borders(Borders::ALL);
    let paragraph = Paragraph::new(app.logs.as_str())
        .block(block)
        .wrap(Wrap { trim: true });
    let area = centered_rect(60, 50, f.area());
    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
}

fn render_agents_list(f: &mut Frame, app: &mut App) {
    let items: Vec<ListItem> = app
        .agents
        .iter()
        .map(|a| ListItem::new(format!("{}: {}", a.id, a.system_prompt)))
        .collect();
    let list = List::new(items).block(Block::default().title("Agents").borders(Borders::ALL));
    let area = centered_rect(60, 25, f.area());
    f.render_widget(Clear, area);
    f.render_widget(list, area);
}

fn render_okrs(f: &mut Frame, app: &mut App) {
    let mut lines = Vec::new();
    for okr in &app.okrs {
        lines.push(Line::from(Span::styled(
            &okr.objective,
            Style::default().add_modifier(Modifier::BOLD),
        )));
        for kr in &okr.key_results {
            lines.push(Line::from(format!(
                " - {} ({:.0}%)",
                kr.name,
                kr.progress * 100.0
            )));
        }
        lines.push(Line::raw(""));
    }
    let block = Block::default().title("OKRs").borders(Borders::ALL);
    let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    let area = centered_rect(60, 50, f.area());
    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
}

fn render_commands(f: &mut Frame) {
    let lines = vec![
        Line::from("q - Quit"),
        Line::from("←/→ or Tab - Switch columns"),
        Line::from("↑/↓ - Navigate tasks"),
        Line::from("h/l - Move task"),
        Line::from("n - New task"),
        Line::from("u - Edit task"),
        Line::from("d - Delete task"),
        Line::from("a - Assign agent"),
        Line::from("c - Add comment"),
        Line::from("L - View logs"),
        Line::from("A - List agents"),
        Line::from("O - Show OKRs"),
    ];
    let block = Block::default().title("Commands").borders(Borders::ALL);
    let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
    let area = centered_rect(60, 50, f.area());
    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

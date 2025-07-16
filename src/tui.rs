use crate::agent::{self, Agent};
use crate::store::{self, Board, Task, TaskStatus};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};
use std::io;

enum View {
    Board,
    TaskDescription,
    AssignAgent,
}

struct App {
    board: Board,
    agents: Vec<Agent>,
    selected_column: usize,
    selected_task: [ListState; 3],
    current_view: View,
    agent_list_state: ListState,
}

impl App {
    fn new(board: Board, agents: Vec<Agent>) -> Self {
        let mut app = App {
            board,
            agents,
            selected_column: 0,
            selected_task: [
                ListState::default(),
                ListState::default(),
                ListState::default(),
            ],
            current_view: View::Board,
            agent_list_state: ListState::default(),
        };
        app.selected_task[0].select(Some(0));
        app
    }

    fn next_column(&mut self) {
        self.selected_column = (self.selected_column + 1) % 3;
    }

    fn prev_column(&mut self) {
        self.selected_column = (self.selected_column + 2) % 3;
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

    fn tasks_in_current_column(&self) -> Vec<&Task> {
        let status = match self.selected_column {
            0 => TaskStatus::ToDo,
            1 => TaskStatus::InProgress,
            _ => TaskStatus::Done,
        };
        self.board
            .tasks
            .iter()
            .filter(|t| t.status == status)
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
            if let Some(task) = self.board.tasks.iter_mut().find(|t| t.id == task_id) {
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

    fn get_selected_task(&self) -> Option<&Task> {
        self.selected_task[self.selected_column]
            .selected()
            .and_then(|selected_index| {
                let tasks_in_column = self.tasks_in_current_column();
                tasks_in_column.get(selected_index).copied()
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
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match app.current_view {
                View::Board => match key.code {
                    KeyCode::Char('q') => {
                        store::save_board(&app.board).unwrap();
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
                                if let Some(task_id) = app.get_selected_task().map(|t| t.id) {
                                    if let Some(task) =
                                        app.board.tasks.iter_mut().find(|t| t.id == task_id)
                                    {
                                        task.agent_id = Some(agent.id);
                                        // `execute_task` is asynchronous. We are inside the synchronous
                                        // `run_app` loop which itself is executed from an async Tokio
                                        // runtime (the `#[tokio::main]` in `main.rs`).  We therefore
                                        // leverage the currently-running runtime handle to synchronously
                                        // wait on the future. Using `Handle::current().block_on(...)` keeps
                                        // the API here synchronous without spinning up a brand-new runtime
                                        // each time.
                                        // Calling `Handle::current().block_on(...)` inside an already
                                        // running Tokio runtime panics ("Cannot start a runtime from
                                        // within a runtime").  To remain in the synchronous context of
                                        // the TUI while still awaiting the async `execute_task` call we
                                        // off-load the blocking operation to Tokio's *blocking* thread
                                        // pool via `block_in_place`.  Once on the dedicated blocking
                                        // thread we spin up a lightweight *current-thread* runtime just
                                        // for the duration of the task execution.
                                        match tokio::task::block_in_place(|| {
                                            tokio::runtime::Builder::new_current_thread()
                                                .enable_all()
                                                .build()
                                                .unwrap()
                                                .block_on(agent::execute_task(&agent, task))
                                        }) {
                                            Ok(result) => match result {
                                                agent::ExecutionResult::Success => {
                                                    task.status = store::TaskStatus::Done;
                                                    task.comment = Some(
                                                        "Task completed successfully.".to_string(),
                                                    );
                                                }
                                                agent::ExecutionResult::Failure { comment } => {
                                                    task.status = store::TaskStatus::ToDo;
                                                    task.comment = Some(comment);
                                                    task.agent_id = None;
                                                }
                                            },
                                            Err(_) => {
                                                task.status = store::TaskStatus::ToDo;
                                                task.comment =
                                                    Some("Failed to execute task.".to_string());
                                                task.agent_id = None;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        store::save_board(&app.board).unwrap();
                        app.current_view = View::Board;
                    }
                    _ => {}
                },
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    render_board(f, app);
    match app.current_view {
        View::TaskDescription => render_task_description(f, app),
        View::AssignAgent => render_assign_agent(f, app),
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
        .split(f.size());

    for (i, status) in [TaskStatus::ToDo, TaskStatus::InProgress, TaskStatus::Done]
        .iter()
        .enumerate()
    {
        let tasks: Vec<ListItem> = app
            .board
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
                .title(format!("{:?}", status))
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
            text.push(Line::from(format!("Assigned to agent: {}", agent_id)));
        }

        if let Some(comment) = &task.comment {
            text.push(Line::from(Span::styled(
                format!("Comment: {}", comment),
                Style::default().fg(Color::Yellow),
            )));
        }

        let block = Block::default()
            .title("Task Description")
            .borders(Borders::ALL);
        let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
        let area = centered_rect(60, 25, f.size());
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
        let area = centered_rect(60, 25, f.size());
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

    let area = centered_rect(60, 25, f.size());
    f.render_widget(Clear, area);
    f.render_stateful_widget(agent_list, area, &mut app.agent_list_state);
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

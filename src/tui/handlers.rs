use super::app::{App, View};
use super::render::ui;
use crate::agent::{self};
use crate::store::{self, Task, TaskStatus};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use notify::{recommended_watcher, RecursiveMode, Watcher};
use ratatui::prelude::*;
use std::io;
use std::path::Path;
use std::sync::{mpsc::channel, Arc};
use std::time::Duration;

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
    let (tx, rx) = channel();
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
                                app.popup_scroll = 0;
                            }
                        }
                        KeyCode::Char('c') => {
                            if app.get_selected_task().is_some() {
                                app.current_view = View::AddComment;
                                app.comment_input.clear();
                                app.popup_scroll = 0;
                            }
                        }
                        KeyCode::Char('n') => {
                            app.new_task_title.clear();
                            app.new_task_description.clear();
                            app.editing_description = false;
                            app.current_view = View::AddTask;
                            app.popup_scroll = 0;
                        }
                        KeyCode::Char('u') => {
                            if let Some(task) = app.get_selected_task() {
                                app.new_task_title = task.title;
                                app.new_task_description = task.description.unwrap_or_default();
                                app.editing_description = false;
                                app.current_view = View::UpdateTask;
                                app.popup_scroll = 0;
                            }
                        }
                        KeyCode::Char('r') => {
                            app.unassign_selected_task();
                            store::save_board(&app.board.lock().unwrap()).unwrap();
                        }
                        KeyCode::Char('d') => {
                            if let Some(task_id) = app.get_selected_task().map(|t| t.id) {
                                app.board.lock().unwrap().tasks.retain(|t| t.id != task_id);
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
                            app.popup_scroll = 0;
                        }
                        KeyCode::Char('A') => {
                            app.agents = crate::agent::load_agents().unwrap_or_default();
                            app.current_view = View::Agents;
                            app.popup_scroll = 0;
                        }
                        KeyCode::Char('O') => {
                            app.okrs = store::load_okrs().unwrap_or_default();
                            app.current_view = View::Okrs;
                            app.popup_scroll = 0;
                        }
                        KeyCode::Char('?') => {
                            app.current_view = View::Commands;
                            app.popup_scroll = 0;
                        }
                        _ => {}
                    },
                    View::TaskDescription => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            app.current_view = View::Board;
                            app.popup_scroll = 0;
                        }
                        KeyCode::Down => {
                            app.popup_scroll = app.popup_scroll.saturating_add(1);
                        }
                        KeyCode::Up => {
                            app.popup_scroll = app.popup_scroll.saturating_sub(1);
                        }
                        _ => {}
                    },
                    View::AssignAgent => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            app.current_view = View::Board;
                            app.popup_scroll = 0;
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
                                        if let Some(task_to_update) =
                                            board.tasks.iter_mut().find(|t| t.id == task.id)
                                        {
                                            task_to_update.agent_id = Some(agent.id);
                                        }
                                        let agent_clone = agent.clone();
                                        let task_clone = task.clone();
                                        let board_clone = Arc::clone(&app.board);
                                        tokio::spawn(async move {
                                            let result = agent::execute_task(
                                                &agent_clone,
                                                Some(&task_clone),
                                            )
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
                            app.popup_scroll = 0;
                        }
                        KeyCode::Enter => {
                            if let Some(task_id) = app.get_selected_task().map(|t| t.id) {
                                if let Some(task) = app
                                    .board
                                    .lock()
                                    .unwrap()
                                    .tasks
                                    .iter_mut()
                                    .find(|t| t.id == task_id)
                                {
                                    task.comment = Some(app.comment_input.clone());
                                }
                                store::save_board(&app.board.lock().unwrap()).unwrap();
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
                                let new_id = app.board.lock().unwrap().tasks.len() + 1;
                                let task = Task {
                                    id: new_id,
                                    title: app.new_task_title.clone(),
                                    description: if app.new_task_description.is_empty() {
                                        None
                                    } else {
                                        Some(app.new_task_description.clone())
                                    },
                                    status: TaskStatus::ToDo,
                                    agent_id: None,
                                    comment: None,
                                };
                                app.board.lock().unwrap().tasks.push(task);
                                store::save_board(&app.board.lock().unwrap()).unwrap();
                                app.current_view = View::Board;
                                app.popup_scroll = 0;
                                app.editing_description = false;
                            } else {
                                app.editing_description = true;
                            }
                        }
                        KeyCode::Esc => {
                            app.current_view = View::Board;
                            app.popup_scroll = 0;
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
                                    if let Some(task) = app
                                        .board
                                        .lock()
                                        .unwrap()
                                        .tasks
                                        .iter_mut()
                                        .find(|t| t.id == task_id)
                                    {
                                        task.title = app.new_task_title.clone();
                                        task.description = if app.new_task_description.is_empty() {
                                            None
                                        } else {
                                            Some(app.new_task_description.clone())
                                        };
                                    }
                                    store::save_board(&app.board.lock().unwrap()).unwrap();
                                }
                                app.current_view = View::Board;
                                app.editing_description = false;
                            } else {
                                app.editing_description = true;
                            }
                        }
                        KeyCode::Esc => {
                            app.current_view = View::Board;
                            app.popup_scroll = 0;
                            app.editing_description = false;
                        }
                        _ => {}
                    },
                    View::Logs | View::Agents | View::Okrs | View::Commands => match key.code {
                        KeyCode::Char('q') | KeyCode::Esc | KeyCode::Char('?') => {
                            app.current_view = View::Board;
                            app.popup_scroll = 0;
                        }
                        KeyCode::Down => {
                            app.popup_scroll = app.popup_scroll.saturating_add(1);
                        }
                        KeyCode::Up => {
                            app.popup_scroll = app.popup_scroll.saturating_sub(1);
                        }
                        _ => {}
                    },
                }
            }
        }
    }
}

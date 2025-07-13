use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState},
};
use std::io;
use crate::store::{self, Board, Task, TaskStatus};

struct App {
    board: Board,
    selected_column: usize,
    selected_task: [ListState; 3],
}

impl App {
    fn new(board: Board) -> Self {
        let mut app = App {
            board,
            selected_column: 0,
            selected_task: [ListState::default(), ListState::default(), ListState::default()],
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
        self.board.tasks.iter().filter(|t| t.status == status).collect()
    }

    fn move_task_to_next_column(&mut self) {
        self.move_task(1);
    }

    fn move_task_to_prev_column(&mut self) {
        self.move_task(-1);
    }

    fn move_task(&mut self, direction: i8) {
        let task_id_to_move = if let Some(selected_index) = self.selected_task[self.selected_column].selected() {
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
}

pub fn run_tui() -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let board = store::load_board().unwrap_or_default();
    let app = App::new(board);
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
            match key.code {
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
                _ => {}
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(33), Constraint::Percentage(33), Constraint::Percentage(34)].as_ref())
        .split(f.size());

    for (i, status) in [TaskStatus::ToDo, TaskStatus::InProgress, TaskStatus::Done].iter().enumerate() {
        let tasks: Vec<ListItem> = app.board.tasks.iter().filter(|t| t.status == *status).map(|t| ListItem::new(t.title.as_str())).collect();
        let mut list = List::new(tasks).block(Block::default().title(format!("{:?}", status)).borders(Borders::ALL));
        if app.selected_column == i {
            list = list.highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(Color::Blue));
        }
        f.render_stateful_widget(list, chunks[i], &mut app.selected_task[i]);
    }
}

use crate::agent::Agent;
use crate::store::{self, Board, Okr, Task, TaskStatus};
use ratatui::widgets::ListState;
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy)]
pub enum View {
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

pub struct App {
    pub board: Arc<Mutex<Board>>,
    pub agents: Vec<Agent>,
    pub selected_column: usize,
    pub selected_task: [ListState; 3],
    pub current_view: View,
    pub agent_list_state: ListState,
    pub comment_input: String,
    pub new_task_title: String,
    pub new_task_description: String,
    pub editing_description: bool,
    pub logs: String,
    pub okrs: Vec<Okr>,
    pub popup_scroll: u16,
}

impl App {
    pub fn new(board: Board, agents: Vec<Agent>) -> Self {
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
            popup_scroll: 0,
        };
        app.selected_task[0].select(Some(0));
        app
    }

    pub fn next_column(&mut self) {
        self.selected_column = (self.selected_column + 1) % 3;
        self.ensure_selected_task();
    }

    pub fn prev_column(&mut self) {
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

    pub fn next_task(&mut self) {
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

    pub fn prev_task(&mut self) {
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

    pub fn tasks_in_current_column(&self) -> Vec<Task> {
        let status = match self.selected_column {
            0 => TaskStatus::ToDo,
            1 => TaskStatus::InProgress,
            _ => TaskStatus::Done,
        };
        self.board
            .lock()
            .expect("board mutex poisoned")
            .tasks
            .iter()
            .filter(|t| t.status == status)
            .cloned()
            .collect()
    }

    pub fn move_task_to_next_column(&mut self) {
        self.move_task(1);
    }

    pub fn move_task_to_prev_column(&mut self) {
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
                .expect("board mutex poisoned")
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

            // Adjust selection if the task moved out of the current column
            let tasks_left = self.tasks_in_current_column();
            if tasks_left.is_empty() {
                self.selected_task[self.selected_column].select(None);
            } else if let Some(idx) = self.selected_task[self.selected_column].selected() {
                if idx >= tasks_left.len() {
                    self.selected_task[self.selected_column].select(Some(tasks_left.len() - 1));
                }
            }
        }
    }

    pub fn get_selected_task(&self) -> Option<Task> {
        self.selected_task[self.selected_column]
            .selected()
            .and_then(|selected_index| {
                let tasks_in_column = self.tasks_in_current_column();
                tasks_in_column.get(selected_index).cloned()
            })
    }
}

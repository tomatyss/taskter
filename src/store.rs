use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum TaskStatus {
    ToDo,
    InProgress,
    Done,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Task {
    pub id: usize,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub agent_id: Option<usize>,
    pub comment: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
pub struct Board {
    pub tasks: Vec<Task>,
}

impl Board {
    fn next_id(&self) -> usize {
        self.tasks.iter().map(|t| t.id).max().unwrap_or(0) + 1
    }

    pub fn add_task(&mut self, title: String, description: Option<String>) -> usize {
        let id = self.next_id();
        let task = Task {
            id,
            title,
            description,
            status: TaskStatus::ToDo,
            agent_id: None,
            comment: None,
        };
        self.tasks.push(task);
        id
    }

    pub fn mark_done(&mut self, id: usize) -> bool {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
            task.status = TaskStatus::Done;
            true
        } else {
            false
        }
    }

    pub fn assign_agent(&mut self, task_id: usize, agent_id: usize) -> bool {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.agent_id = Some(agent_id);
            true
        } else {
            false
        }
    }

    pub fn add_comment(&mut self, task_id: usize, comment: String) -> bool {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == task_id) {
            task.comment = Some(comment);
            true
        } else {
            false
        }
    }

    pub fn update_task(&mut self, id: usize, title: String, description: Option<String>) -> bool {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
            task.title = title;
            task.description = description;
            true
        } else {
            false
        }
    }

    pub fn remove_task(&mut self, id: usize) -> bool {
        let initial = self.tasks.len();
        self.tasks.retain(|t| t.id != id);
        initial != self.tasks.len()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct KeyResult {
    pub name: String,
    pub progress: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Okr {
    pub objective: String,
    pub key_results: Vec<KeyResult>,
}

pub fn load_board() -> anyhow::Result<Board> {
    let path = Path::new(".taskter/board.json");
    if !path.exists() {
        return Ok(Board::default());
    }

    let content = fs::read_to_string(path)?;
    let board: Board = serde_json::from_str(&content)?;
    Ok(board)
}

pub fn save_board(board: &Board) -> anyhow::Result<()> {
    let path = Path::new(".taskter/board.json");
    let content = serde_json::to_string_pretty(board)?;
    fs::write(path, content)?;
    Ok(())
}

pub fn load_okrs() -> anyhow::Result<Vec<Okr>> {
    let path = Path::new(".taskter/okrs.json");
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(path)?;
    let okrs: Vec<Okr> = serde_json::from_str(&content)?;
    Ok(okrs)
}

pub fn save_okrs(okrs: &[Okr]) -> anyhow::Result<()> {
    let path = Path::new(".taskter/okrs.json");
    let content = serde_json::to_string_pretty(okrs)?;
    fs::write(path, content)?;
    Ok(())
}

use serde::{Deserialize, Serialize};
use std::fs;

use crate::config;

/// Progress state of a [`Task`].
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum TaskStatus {
    ToDo,
    InProgress,
    Done,
}

/// A single task stored in `.taskter/board.json`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Task {
    pub id: usize,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub agent_id: Option<usize>,
    pub comment: Option<String>,
}

/// Collection of tasks comprising the Kanban board.
#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
pub struct Board {
    pub tasks: Vec<Task>,
}

impl Board {
    /// Returns the next available unique task ID.
    pub fn next_task_id(&self) -> usize {
        self.tasks.iter().map(|t| t.id).max().unwrap_or(0) + 1
    }
}

/// A measurable key result belonging to an [`Okr`].
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct KeyResult {
    pub name: String,
    pub progress: f32,
}

/// Objective with its associated key results stored in `.taskter/okrs.json`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Okr {
    pub objective: String,
    pub key_results: Vec<KeyResult>,
}

/// Reads the Kanban board from `.taskter/board.json`.
///
/// Returns an empty board if the file does not exist.
pub fn load_board() -> anyhow::Result<Board> {
    let path = config::board_path();
    if !path.exists() {
        return Ok(Board::default());
    }

    let content = fs::read_to_string(path)?;
    let board: Board = serde_json::from_str(&content)?;
    Ok(board)
}

/// Writes the current board state to `.taskter/board.json`.
pub fn save_board(board: &Board) -> anyhow::Result<()> {
    let path = config::board_path();
    let content = serde_json::to_string_pretty(board)?;
    fs::write(path, content)?;
    Ok(())
}

/// Loads all OKRs from `.taskter/okrs.json`.
///
/// Returns an empty list if the file is missing.
pub fn load_okrs() -> anyhow::Result<Vec<Okr>> {
    let path = config::okrs_path();
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(path)?;
    let okrs: Vec<Okr> = serde_json::from_str(&content)?;
    Ok(okrs)
}

/// Persists OKRs to `.taskter/okrs.json`.
pub fn save_okrs(okrs: &[Okr]) -> anyhow::Result<()> {
    let path = config::okrs_path();
    let content = serde_json::to_string_pretty(okrs)?;
    fs::write(path, content)?;
    Ok(())
}

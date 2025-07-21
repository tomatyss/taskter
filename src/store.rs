use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Status of a task on the board.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum TaskStatus {
    /// The task has not been started yet.
    ToDo,
    /// The task is currently in progress.
    InProgress,
    /// The task is finished.
    Done,
}

/// A single task entry stored in `.taskter/board.json`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Task {
    /// Unique identifier of the task.
    pub id: usize,
    /// Short task title displayed in the board.
    pub title: String,
    /// Optional longer description.
    pub description: Option<String>,
    /// Current status of the task.
    pub status: TaskStatus,
    /// Identifier of the assigned agent, if any.
    pub agent_id: Option<usize>,
    /// Comment returned by an agent after execution.
    pub comment: Option<String>,
}

/// Top level container for all tasks.
#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
pub struct Board {
    /// All tasks recorded in the board.
    pub tasks: Vec<Task>,
}

/// A single key result belonging to an OKR.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct KeyResult {
    /// Descriptive name of the key result.
    pub name: String,
    /// Progress value between 0 and 1.
    pub progress: f32,
}

/// Objective and its associated key results.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Okr {
    /// Objective description.
    pub objective: String,
    /// Key results tracking the objective.
    pub key_results: Vec<KeyResult>,
}

/// Load the task board from `.taskter/board.json`.
///
/// If the file is missing an empty board is returned.
pub fn load_board() -> anyhow::Result<Board> {
    let path = Path::new(".taskter/board.json");
    if !path.exists() {
        return Ok(Board::default());
    }

    let content = fs::read_to_string(path)?;
    let board: Board = serde_json::from_str(&content)?;
    Ok(board)
}

/// Save the current board state to `.taskter/board.json`.
pub fn save_board(board: &Board) -> anyhow::Result<()> {
    let path = Path::new(".taskter/board.json");
    let content = serde_json::to_string_pretty(board)?;
    fs::write(path, content)?;
    Ok(())
}

/// Load OKRs from `.taskter/okrs.json`.
///
/// Returns an empty list if the file does not exist.
pub fn load_okrs() -> anyhow::Result<Vec<Okr>> {
    let path = Path::new(".taskter/okrs.json");
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(path)?;
    let okrs: Vec<Okr> = serde_json::from_str(&content)?;
    Ok(okrs)
}

/// Persist the provided OKRs to `.taskter/okrs.json`.
pub fn save_okrs(okrs: &[Okr]) -> anyhow::Result<()> {
    let path = Path::new(".taskter/okrs.json");
    let content = serde_json::to_string_pretty(okrs)?;
    fs::write(path, content)?;
    Ok(())
}

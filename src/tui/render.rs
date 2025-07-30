use super::app::{App, View};
use crate::store::TaskStatus;
use ratatui::{
    prelude::*,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};

pub(crate) fn ui(f: &mut Frame, app: &mut App) {
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
        View::Commands => render_commands(f, app),
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
        let paragraph = Paragraph::new(text)
            .block(block)
            .wrap(Wrap { trim: true })
            .scroll((app.popup_scroll, 0));
        let area = centered_rect(60, 25, f.area());
        f.render_widget(Clear, area);
        f.render_widget(paragraph, area);
    }
}

fn render_assign_agent(f: &mut Frame, app: &mut App) {
    if app.agents.is_empty() {
        let block = Block::default().title("Assign Agent").borders(Borders::ALL);
        let text = Paragraph::new("No agents available. Create one with `taskter agent add`")
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
        .wrap(Wrap { trim: true })
        .scroll((app.popup_scroll, 0));
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
    .wrap(Wrap { trim: true })
    .scroll((app.popup_scroll, 0));
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
    .wrap(Wrap { trim: true })
    .scroll((app.popup_scroll, 0));
    let area = centered_rect(60, 15, f.area());
    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
}

fn render_logs(f: &mut Frame, app: &mut App) {
    let block = Block::default().title("Logs").borders(Borders::ALL);
    let paragraph = Paragraph::new(app.logs.as_str())
        .block(block)
        .wrap(Wrap { trim: true })
        .scroll((app.popup_scroll, 0));
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
    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: true })
        .scroll((app.popup_scroll, 0));
    let area = centered_rect(60, 50, f.area());
    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
}

fn render_commands(f: &mut Frame, app: &mut App) {
    let lines = vec![
        Line::from("q - Quit"),
        Line::from("←/→ or Tab - Switch columns"),
        Line::from("↑/↓ - Navigate tasks"),
        Line::from("h/l - Move task"),
        Line::from("n - New task"),
        Line::from("u - Edit task"),
        Line::from("d - Delete task"),
        Line::from("a - Assign agent"),
        Line::from("r - Unassign agent"),
        Line::from("c - Add comment"),
        Line::from("L - View logs"),
        Line::from("A - List agents"),
        Line::from("O - Show OKRs"),
    ];
    let block = Block::default().title("Commands").borders(Borders::ALL);
    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: true })
        .scroll((app.popup_scroll, 0));
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

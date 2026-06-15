use std::cell::RefCell;

use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::ListItem;

use crate::config::Theme;
use crate::model::Model;
use crate::model::commit::{Commit, CommitStatus};

use super::graph;

/// Cached commit-graph layout. `graph::compute_graph` is O(commits × lanes) and
/// allocates a `Vec<Cell>` per row plus per-lane `String` clones, so recomputing
/// it on every redraw (every scroll/keystroke in the commits view) is wasteful.
/// We recompute only when the commit list actually changes, keyed by a cheap
/// `(len, first-hash, last-hash)` signature: rewriting history changes the tip
/// hash, and pagination / refresh / branch-filtering change the length or
/// endpoints, so this catches every real change while making the steady-state
/// redraw a plain borrow of the cached rows.
#[derive(Default)]
struct GraphCacheEntry {
    len: usize,
    first_hash: String,
    last_hash: String,
    rows: Vec<graph::GraphRow>,
    max_width: usize,
}

impl GraphCacheEntry {
    fn refresh(&mut self, commits: &[Commit]) {
        let len = commits.len();
        let first = commits.first().map(|c| c.hash.as_str()).unwrap_or("");
        let last = commits.last().map(|c| c.hash.as_str()).unwrap_or("");
        if self.len == len && self.first_hash == first && self.last_hash == last {
            return; // unchanged — reuse the cached rows
        }
        let graph_input: Vec<(String, Vec<String>)> = commits
            .iter()
            .map(|c| (c.hash.clone(), c.parents.clone()))
            .collect();
        self.rows = graph::compute_graph(&graph_input);
        self.max_width = self.rows.iter().map(|r| r.cells.len()).max().unwrap_or(0);
        self.len = len;
        self.first_hash = first.to_string();
        self.last_hash = last.to_string();
    }
}

thread_local! {
    /// Slot 0 = main commit list, slot 1 = sub-commit list (branch/remote
    /// commits view), so alternating between the two views doesn't thrash.
    static GRAPH_CACHE: RefCell<[GraphCacheEntry; 2]> =
        RefCell::new([GraphCacheEntry::default(), GraphCacheEntry::default()]);
}

pub fn render_sub_commit_list<'a>(model: &Model, theme: &Theme) -> Vec<ListItem<'a>> {
    render_commits(&model.sub_commits, &model.head_hash, theme, &[], 1)
}

pub fn render_commit_list<'a>(
    model: &Model,
    theme: &Theme,
    cherry_picked: &[String],
) -> Vec<ListItem<'a>> {
    render_commits(&model.commits, &model.head_hash, theme, cherry_picked, 0)
}

fn render_commits<'a>(
    commits: &[Commit],
    head_hash: &str,
    theme: &Theme,
    cherry_picked: &[String],
    slot: usize,
) -> Vec<ListItem<'a>> {
    GRAPH_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        let entry = &mut cache[slot];
        // Recompute the graph layout only when the commit list changed.
        entry.refresh(commits);
        let max_graph_width = entry.max_width;
        let graph_rows = &entry.rows;

        commits
            .iter()
            .enumerate()
            .map(|(i, commit)| {
                let graph_row = graph_rows.get(i);
                let is_head = commit.hash == *head_hash;

                // Start with graph spans.
                let mut spans: Vec<Span<'a>> = if let Some(row) = graph_row {
                    graph::render_graph_spans(row, max_graph_width, is_head, theme)
                } else {
                    vec![Span::raw(" ".repeat(max_graph_width * 2))]
                };

                // Hash — color by push status, overridden to cyan+bold if cherry-picked
                let is_cherry_picked = cherry_picked.contains(&commit.hash);
                let hash_style = if is_cherry_picked {
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD)
                } else {
                    match commit.status {
                        CommitStatus::Unpushed => theme.commit_hash,
                        CommitStatus::Pushed => Style::default().fg(theme.commit_hash_pushed),
                        CommitStatus::Merged => Style::default().fg(theme.commit_hash_merged),
                        _ => theme.commit_hash,
                    }
                };
                spans.push(Span::styled(
                    format!("{} ", commit.short_hash()),
                    hash_style,
                ));

                // Ref decorations (HEAD -> main, origin/main, etc.)
                for r in &commit.refs {
                    #[allow(clippy::if_same_then_else)]
                    let (label, color) = if r.starts_with("HEAD -> ") {
                        (r.clone(), theme.ref_head)
                    } else if r == "HEAD" {
                        (r.clone(), theme.ref_head)
                    } else if r.contains('/') {
                        // Remote ref like origin/main
                        (r.clone(), theme.ref_remote)
                    } else {
                        // Local branch
                        (r.clone(), theme.ref_local)
                    };
                    spans.push(Span::styled(
                        format!("({}) ", label),
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ));
                }

                // Tags (before message so they're visible in compact views)
                for tag in &commit.tags {
                    spans.push(Span::styled(
                        format!("[{}] ", tag),
                        Style::default()
                            .fg(theme.ref_tag)
                            .add_modifier(Modifier::BOLD),
                    ));
                }

                // Commit message
                spans.push(Span::styled(
                    commit.name.clone(),
                    Style::default().fg(theme.text_strong),
                ));

                // Author (compact)
                spans.push(Span::styled(
                    format!(" {}", commit.author_name),
                    theme.commit_author,
                ));

                ListItem::new(Line::from(spans))
            })
            .collect()
    })
}

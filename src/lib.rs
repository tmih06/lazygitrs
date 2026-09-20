pub mod app;
pub mod config;
pub mod git;
pub mod gui;
pub mod model;
pub mod os;
pub mod pager;
pub mod upgrade;

use std::path::PathBuf;

/// Embedded entry point: runs the TUI on `repo_path` until the user quits.
/// Sets up and tears down its own terminal; the caller must NOT hold the
/// terminal (raw mode / alt screen) while this runs.
pub fn run(repo_path: PathBuf, debug: bool, filter_path: Option<PathBuf>) -> anyhow::Result<()> {
    app::App::new(repo_path, debug, filter_path)?.run()
}

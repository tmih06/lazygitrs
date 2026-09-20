use std::sync::OnceLock;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use serde::{Deserialize, Serialize};

/// A keybinding spec (e.g. `"q"`, `"<c-c>"`, `"<enter>"`) that parses its raw
/// string into a [`KeyEvent`] exactly once, caching the result. Previously
/// every keypress re-parsed every binding string via [`parse_key`]; now the
/// parse happens on first use and subsequent matches are a plain integer
/// comparison of the cached event.
///
/// `Deref<Target = str>` and the `From`/serde impls keep it a drop-in for the
/// `String` it replaced (serializes/deserializes transparently as a string).
#[derive(Debug, Clone, Default)]
pub struct Key {
    raw: String,
    parsed: OnceLock<Option<KeyEvent>>,
}

impl Key {
    pub fn new(raw: String) -> Self {
        Self {
            raw,
            parsed: OnceLock::new(),
        }
    }

    /// The parsed key event, computed once and cached for the lifetime of the
    /// binding.
    pub fn event(&self) -> Option<KeyEvent> {
        *self.parsed.get_or_init(|| parse_key(&self.raw))
    }
}

impl std::ops::Deref for Key {
    type Target = str;
    fn deref(&self) -> &str {
        &self.raw
    }
}

impl From<&str> for Key {
    fn from(s: &str) -> Self {
        Key::new(s.to_string())
    }
}

impl From<String> for Key {
    fn from(s: String) -> Self {
        Key::new(s)
    }
}

impl Serialize for Key {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.raw)
    }
}

impl<'de> Deserialize<'de> for Key {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Key::new(String::deserialize(deserializer)?))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct KeybindingConfig {
    pub universal: UniversalKeybinding,
    pub status: StatusKeybinding,
    pub files: FilesKeybinding,
    pub branches: BranchesKeybinding,
    pub commits: CommitsKeybinding,
    pub stash: StashKeybinding,
    #[serde(rename = "commitMessage")]
    pub commit_message: CommitMessageKeybinding,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UniversalKeybinding {
    pub quit: Key,
    #[serde(rename = "quit-alt1")]
    pub quit_alt1: Key,
    #[serde(rename = "return")]
    pub return_key: Key,
    #[serde(rename = "quitWithoutChangingDirectory")]
    pub quit_without_changing_directory: Key,
    #[serde(rename = "togglePanel")]
    pub toggle_panel: Key,
    #[serde(rename = "togglePanelReverse")]
    pub toggle_panel_reverse: Key,
    #[serde(rename = "prevItem")]
    pub prev_item: Key,
    #[serde(rename = "nextItem")]
    pub next_item: Key,
    #[serde(rename = "prevItem-alt")]
    pub prev_item_alt: Key,
    #[serde(rename = "nextItem-alt")]
    pub next_item_alt: Key,
    #[serde(rename = "prevPage")]
    pub prev_page: Key,
    #[serde(rename = "nextPage")]
    pub next_page: Key,
    #[serde(rename = "scrollLeft")]
    pub scroll_left: Key,
    #[serde(rename = "scrollRight")]
    pub scroll_right: Key,
    #[serde(rename = "gotoTop")]
    pub goto_top: Key,
    #[serde(rename = "gotoBottom")]
    pub goto_bottom: Key,
    #[serde(rename = "prevBlock")]
    pub prev_block: Key,
    #[serde(rename = "nextBlock")]
    pub next_block: Key,
    #[serde(rename = "prevBlock-alt")]
    pub prev_block_alt: Key,
    #[serde(rename = "nextBlock-alt")]
    pub next_block_alt: Key,
    #[serde(rename = "nextMatch")]
    pub next_match: Key,
    #[serde(rename = "prevMatch")]
    pub prev_match: Key,
    #[serde(rename = "startSearch")]
    pub start_search: Key,
    #[serde(rename = "optionMenu")]
    pub option_menu: Key,
    pub edit: Key,
    #[serde(rename = "openFile")]
    pub open_file: Key,
    #[serde(rename = "scrollUpMain")]
    pub scroll_up_main: Key,
    #[serde(rename = "scrollDownMain")]
    pub scroll_down_main: Key,
    #[serde(rename = "scrollUpMain-alt1")]
    pub scroll_up_main_alt1: Key,
    #[serde(rename = "scrollDownMain-alt1")]
    pub scroll_down_main_alt1: Key,
    pub undo: Key,
    pub redo: Key,
    #[serde(rename = "filteringMenu")]
    pub filtering_menu: Key,
    #[serde(rename = "diffingMenu")]
    pub diffing_menu: Key,
    #[serde(rename = "copyToClipboard")]
    pub copy_to_clipboard: Key,
    pub refresh: Key,
    #[serde(rename = "createRebaseOptionsMenu")]
    pub create_rebase_options_menu: Key,
    #[serde(rename = "pushFiles")]
    pub push_files: Key,
    #[serde(rename = "pullFiles")]
    pub pull_files: Key,
    #[serde(rename = "nextScreenMode")]
    pub next_screen_mode: Key,
    #[serde(rename = "prevScreenMode")]
    pub prev_screen_mode: Key,
    #[serde(rename = "createPatchOptionsMenu")]
    pub create_patch_options_menu: Key,
    #[serde(rename = "revertBlock")]
    pub revert_block: Key,
    #[serde(rename = "undoRevertBlock")]
    pub undo_revert_block: Key,
    #[serde(rename = "shrinkSidePanel")]
    pub shrink_side_panel: Key,
    #[serde(rename = "expandSidePanel")]
    pub expand_side_panel: Key,
    #[serde(rename = "sidePanelFull")]
    pub side_panel_full: Key,
    #[serde(rename = "mainPanelFull")]
    pub main_panel_full: Key,
    #[serde(rename = "resetSidePanel")]
    pub reset_side_panel: Key,
    #[serde(rename = "toggleDiffViewLayout")]
    pub toggle_diff_view_layout: Key,
}

impl Default for UniversalKeybinding {
    fn default() -> Self {
        Self {
            quit: "q".into(),
            quit_alt1: "<c-c>".into(),
            return_key: "<escape>".into(),
            quit_without_changing_directory: "Q".into(),
            toggle_panel: "<tab>".into(),
            toggle_panel_reverse: "<backtab>".into(),
            prev_item: "k".into(),
            next_item: "j".into(),
            prev_item_alt: "<up>".into(),
            next_item_alt: "<down>".into(),
            prev_page: "<pgup>".into(),
            next_page: "<pgdown>".into(),
            scroll_left: "H".into(),
            scroll_right: "L".into(),
            goto_top: "<".into(),
            goto_bottom: ">".into(),
            prev_block: "<left>".into(),
            next_block: "<right>".into(),
            prev_block_alt: "h".into(),
            next_block_alt: "l".into(),
            next_match: "n".into(),
            prev_match: "N".into(),
            start_search: "/".into(),
            option_menu: "x".into(),
            edit: "e".into(),
            open_file: "o".into(),
            scroll_up_main: "<pgup>".into(),
            scroll_down_main: "<pgdown>".into(),
            scroll_up_main_alt1: "K".into(),
            scroll_down_main_alt1: "J".into(),
            undo: "z".into(),
            redo: "<c-z>".into(),
            filtering_menu: "<c-s>".into(),
            diffing_menu: "W".into(),
            copy_to_clipboard: "<c-o>".into(),
            refresh: "R".into(),
            create_rebase_options_menu: "m".into(),
            push_files: "P".into(),
            pull_files: "p".into(),
            next_screen_mode: "+".into(),
            prev_screen_mode: "_".into(),
            create_patch_options_menu: "<c-p>".into(),
            revert_block: "<enter>".into(),
            undo_revert_block: "u".into(),
            shrink_side_panel: "<a-h>".into(),
            expand_side_panel: "<a-l>".into(),
            side_panel_full: "<a-k>".into(),
            main_panel_full: "<a-j>".into(),
            reset_side_panel: "<a-r>".into(),
            toggle_diff_view_layout: "\\".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StatusKeybinding {
    #[serde(rename = "checkForUpdate")]
    pub check_for_update: Key,
    #[serde(rename = "recentRepos")]
    pub recent_repos: Key,
    #[serde(rename = "allBranchesLogGraph")]
    pub all_branches_log_graph: Key,
}

impl Default for StatusKeybinding {
    fn default() -> Self {
        Self {
            check_for_update: "u".into(),
            recent_repos: "<enter>".into(),
            all_branches_log_graph: "a".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FilesKeybinding {
    #[serde(rename = "commitChanges")]
    pub commit_changes: Key,
    #[serde(rename = "generateAICommit")]
    pub generate_ai_commit: Key,
    #[serde(rename = "commitChangesWithoutHook")]
    pub commit_changes_without_hook: Key,
    #[serde(rename = "amendLastCommit")]
    pub amend_last_commit: Key,
    #[serde(rename = "commitChangesWithEditor")]
    pub commit_changes_with_editor: Key,
    #[serde(rename = "toggleStagedAll")]
    pub toggle_staged_all: Key,
    #[serde(rename = "stashAllChanges")]
    pub stash_all_changes: Key,
    #[serde(rename = "viewStashOptions")]
    pub view_stash_options: Key,
    #[serde(rename = "toggleTreeView")]
    pub toggle_tree_view: Key,
    #[serde(rename = "toggleFileExplorer")]
    pub toggle_file_explorer: Key,
    pub fetch: Key,
    #[serde(rename = "ignoreFile")]
    pub ignore_file: Key,
}

impl Default for FilesKeybinding {
    fn default() -> Self {
        Self {
            commit_changes: "c".into(),
            generate_ai_commit: "<c-g>".into(),
            commit_changes_without_hook: "w".into(),
            amend_last_commit: "A".into(),
            commit_changes_with_editor: "C".into(),
            toggle_staged_all: "a".into(),
            stash_all_changes: "s".into(),
            view_stash_options: "S".into(),
            toggle_tree_view: "`".into(),
            toggle_file_explorer: "F".into(),
            fetch: "f".into(),
            ignore_file: "i".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BranchesKeybinding {
    #[serde(rename = "createPullRequest")]
    pub create_pull_request: Key,
    #[serde(rename = "viewPullRequestOptions")]
    pub view_pull_request_options: Key,
    #[serde(rename = "checkoutBranchByName")]
    pub checkout_branch_by_name: Key,
    #[serde(rename = "forceCheckoutBranch")]
    pub force_checkout_branch: Key,
    #[serde(rename = "rebaseBranch")]
    pub rebase_branch: Key,
    #[serde(rename = "renameBranch")]
    pub rename_branch: Key,
    #[serde(rename = "mergeIntoCurrentBranch")]
    pub merge_into_current_branch: Key,
    #[serde(rename = "fastForward")]
    pub fast_forward: Key,
    #[serde(rename = "setUpstream")]
    pub set_upstream: Key,
}

impl Default for BranchesKeybinding {
    fn default() -> Self {
        Self {
            create_pull_request: "o".into(),
            view_pull_request_options: "O".into(),
            checkout_branch_by_name: "c".into(),
            force_checkout_branch: "F".into(),
            rebase_branch: "r".into(),
            rename_branch: "R".into(),
            merge_into_current_branch: "M".into(),
            fast_forward: "f".into(),
            set_upstream: "u".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CommitsKeybinding {
    #[serde(rename = "squashDown")]
    pub squash_down: Key,
    #[serde(rename = "renameCommit")]
    pub rename_commit: Key,
    #[serde(rename = "renameCommitWithEditor")]
    pub rename_commit_with_editor: Key,
    #[serde(rename = "viewResetOptions")]
    pub view_reset_options: Key,
    #[serde(rename = "markCommitAsFixup")]
    pub mark_commit_as_fixup: Key,
    #[serde(rename = "createFixupCommit")]
    pub create_fixup_commit: Key,
    #[serde(rename = "squashAboveCommits")]
    pub squash_above_commits: Key,
    #[serde(rename = "moveDownCommit")]
    pub move_down_commit: Key,
    #[serde(rename = "moveUpCommit")]
    pub move_up_commit: Key,
    #[serde(rename = "amendToCommit")]
    pub amend_to_commit: Key,
    #[serde(rename = "pickCommit")]
    pub pick_commit: Key,
    #[serde(rename = "revertCommit")]
    pub revert_commit: Key,
    #[serde(rename = "cherryPickCopy")]
    pub cherry_pick_copy: Key,
    #[serde(rename = "pasteCommits")]
    pub paste_commits: Key,
    #[serde(rename = "tagCommit")]
    pub tag_commit: Key,
    #[serde(rename = "checkoutCommit")]
    pub checkout_commit: Key,
    #[serde(rename = "resetCherryPick")]
    pub reset_cherry_pick: Key,
    #[serde(rename = "openLogMenu")]
    pub open_log_menu: Key,
    #[serde(rename = "viewBisectOptions")]
    pub view_bisect_options: Key,
    #[serde(rename = "interactiveRebase")]
    pub interactive_rebase: Key,
}

impl Default for CommitsKeybinding {
    fn default() -> Self {
        Self {
            squash_down: "s".into(),
            rename_commit: "r".into(),
            rename_commit_with_editor: "R".into(),
            view_reset_options: "g".into(),
            mark_commit_as_fixup: "f".into(),
            create_fixup_commit: "F".into(),
            squash_above_commits: "S".into(),
            move_down_commit: "<c-j>".into(),
            move_up_commit: "<c-k>".into(),
            amend_to_commit: "A".into(),
            pick_commit: "p".into(),
            revert_commit: "t".into(),
            cherry_pick_copy: "C".into(),
            paste_commits: "V".into(),
            tag_commit: "T".into(),
            checkout_commit: "<space>".into(),
            reset_cherry_pick: "<c-q>".into(),
            open_log_menu: "<c-s>".into(),
            view_bisect_options: "b".into(),
            interactive_rebase: "i".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StashKeybinding {
    #[serde(rename = "popStash")]
    pub pop_stash: Key,
    #[serde(rename = "renameStash")]
    pub rename_stash: Key,
}

impl Default for StashKeybinding {
    fn default() -> Self {
        Self {
            pop_stash: "g".into(),
            rename_stash: "r".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CommitMessageKeybinding {
    #[serde(rename = "commitMenu")]
    pub commit_menu: Key,
    #[serde(rename = "aiGenerate")]
    pub ai_generate: Key,
}

impl Default for CommitMessageKeybinding {
    fn default() -> Self {
        Self {
            commit_menu: "<c-o>".into(),
            ai_generate: "<c-g>".into(),
        }
    }
}

pub fn parse_key(s: &str) -> Option<KeyEvent> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    if s.starts_with('<') && s.ends_with('>') {
        let inner = &s[1..s.len() - 1];

        if let Some(key) = inner.strip_prefix("c-") {
            let ch = key.chars().next()?;
            return Some(KeyEvent::new(KeyCode::Char(ch), KeyModifiers::CONTROL));
        }

        if let Some(key) = inner.strip_prefix("a-") {
            let ch = key.chars().next()?;
            return Some(KeyEvent::new(KeyCode::Char(ch), KeyModifiers::ALT));
        }

        return match inner {
            "enter" => Some(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
            "escape" | "esc" => Some(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
            "tab" => Some(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)),
            "backtab" | "shift-tab" => Some(KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT)),
            "backspace" | "bs" => Some(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE)),
            "delete" | "del" => Some(KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE)),
            "space" => Some(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE)),
            "up" => Some(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)),
            "down" => Some(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)),
            "left" => Some(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE)),
            "right" => Some(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE)),
            "pgup" => Some(KeyEvent::new(KeyCode::PageUp, KeyModifiers::NONE)),
            "pgdown" => Some(KeyEvent::new(KeyCode::PageDown, KeyModifiers::NONE)),
            "home" => Some(KeyEvent::new(KeyCode::Home, KeyModifiers::NONE)),
            "end" => Some(KeyEvent::new(KeyCode::End, KeyModifiers::NONE)),
            _ => None,
        };
    }

    if s.len() == 1 {
        let ch = s.chars().next()?;
        let modifiers = if ch.is_uppercase() {
            KeyModifiers::SHIFT
        } else {
            KeyModifiers::NONE
        };
        return Some(KeyEvent::new(KeyCode::Char(ch), modifiers));
    }

    None
}

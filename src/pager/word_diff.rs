use similar::{ChangeTag, TextDiff};

use super::InlineSegment;

/// Check if a string contains meaningful (non-whitespace) content.
fn has_meaningful_content(s: &str) -> bool {
    s.chars().any(|c| !c.is_whitespace())
}

/// Compute word-level diff segments for a pair of modified lines.
/// Returns Some((old_segments, new_segments)) if word-level highlighting is useful,
/// or None if the lines are too different to benefit from it.
pub fn compute_word_diff(
    old_text: &str,
    new_text: &str,
) -> Option<(Vec<InlineSegment>, Vec<InlineSegment>)> {
    // Word-level diffing runs Myers under the hood (worst case ~O(n^2)). A
    // single very long line — minified JS/CSS, a base64 blob, a generated
    // lockfile row — can stall the diff for seconds. Past this size the
    // intra-line highlight isn't useful anyway, so fall back to whole-line
    // emphasis by returning None.
    const MAX_WORD_DIFF_LEN: usize = 2000;
    if old_text.len() > MAX_WORD_DIFF_LEN || new_text.len() > MAX_WORD_DIFF_LEN {
        return None;
    }

    let diff = TextDiff::configure().diff_unicode_words(old_text, new_text);

    let mut old_segments = Vec::new();
    let mut new_segments = Vec::new();
    let mut unchanged_len = 0usize;

    for change in diff.iter_all_changes() {
        let text = change.value().to_string();
        match change.tag() {
            ChangeTag::Equal => {
                if has_meaningful_content(&text) {
                    unchanged_len += text.trim().len();
                }
                old_segments.push(InlineSegment {
                    text: text.clone(),
                    emphasized: false,
                });
                new_segments.push(InlineSegment {
                    text,
                    emphasized: false,
                });
            }
            ChangeTag::Delete => {
                old_segments.push(InlineSegment {
                    text,
                    emphasized: true,
                });
            }
            ChangeTag::Insert => {
                new_segments.push(InlineSegment {
                    text,
                    emphasized: true,
                });
            }
        }
    }

    let old_trimmed_len = old_text.trim().len();
    let new_trimmed_len = new_text.trim().len();
    let total_len = old_trimmed_len.max(new_trimmed_len);

    // Only show word-level diff if at least 20% of content is unchanged
    const MIN_UNCHANGED_RATIO: f64 = 0.20;
    if total_len == 0 || (unchanged_len as f64 / total_len as f64) < MIN_UNCHANGED_RATIO {
        return None;
    }

    Some((old_segments, new_segments))
}

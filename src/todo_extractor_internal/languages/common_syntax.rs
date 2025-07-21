// src/languages/common_syntax.rs
//! This module provides common syntax utilities for removing language-specific markers,
//! dedenting multi-line comments, and merging contiguous comment lines.

/// Removes common language-specific comment markers from the beginning and end of the text.
/// It only removes the marker characters (and an optional extra whitespace immediately following
/// a leading marker or preceding a trailing marker) without trimming all other whitespace.
pub fn strip_markers(text: &str) -> String {
    // Work on a mutable owned string.
    let mut result = text.to_string();

    // Remove a leading marker if present.
    // The markers are checked after any initial indentation so that we preserve it.
    let leading_markers = ["<!--", "///", "/*", "//", "#", "--"];
    if let Some(non_ws_idx) = result.find(|c: char| !c.is_whitespace()) {
        for marker in &leading_markers {
            if result[non_ws_idx..].starts_with(marker) {
                let marker_end = non_ws_idx + marker.len();
                // Remove an extra space if it immediately follows the marker.
                let remove_space = if result[marker_end..].starts_with(' ') {
                    1
                } else {
                    0
                };
                result.replace_range(non_ws_idx..(marker_end + remove_space), "");
                break;
            }
        }
    }

    // Remove a trailing marker if present.
    let trailing_markers = ["*/", "-->"];
    for marker in &trailing_markers {
        // First, check for a pattern where there's an extra space before the marker.
        let pattern = format!(" {marker}");
        if result.ends_with(&pattern) {
            let new_len = result.len() - pattern.len();
            result.truncate(new_len);
            break;
        } else if result.ends_with(marker) {
            let new_len = result.len() - marker.len();
            result.truncate(new_len);
            break;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_markers() {
        let input = "/// TODO: Fix this issue";
        let output = strip_markers(input);
        assert_eq!(output, "TODO: Fix this issue");

        let input2 = "/* TODO: Refactor code */";
        let output2 = strip_markers(input2);
        assert_eq!(output2, "TODO: Refactor code");
    }

    #[test]
    fn test_strip_markers_different_markers() {
        let input_hash = "# Note: This is a test";
        assert_eq!(strip_markers(input_hash), "Note: This is a test");

        let input_html = "<!-- Important comment -->";
        assert_eq!(strip_markers(input_html), "Important comment");
    }

    #[test]
    fn test_strip_markers_with_indent() {
        // The indentation before the marker is preserved.
        let input = "    // Indented comment";
        let output = strip_markers(input);
        assert_eq!(output, "    Indented comment");
    }
}

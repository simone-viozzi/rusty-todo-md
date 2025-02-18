// src/languages/common_syntax.rs
//! This module provides common syntax utilities for removing language-specific markers,
//! dedenting multi-line comments, and merging contiguous comment lines.

/// Removes common language-specific comment markers from the beginning and end of the text.
/// For example, it should remove leading `//`, `#`, or `/*` and trailing `*/` if present.
pub fn strip_markers(text: &str) -> String {
    // TODO this function need to preserve the whitespaces.
    //    only remove the markers!

    // Trim whitespace first.
    let trimmed = text.trim();

    // Remove common markers.
    let without_leading = trimmed
        .trim_start_matches("//")
        .trim_start_matches("///")
        .trim_start_matches("#")
        .trim_start_matches("/*")
        .trim_start_matches("<!--"); // In case of HTML or similar

    let without_trailing = without_leading
        .trim_end_matches("*/")
        .trim_end_matches("-->");

    without_trailing.trim().to_string()
}

/// Dedents a multi-line comment by removing the minimum common indentation
/// from all lines. This preserves relative indentation within the comment.
pub fn dedent_comment(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    // Skip empty lines to calculate indent.
    let indent = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.chars().take_while(|c| c.is_whitespace()).count())
        .min()
        .unwrap_or(0);

    // Remove the common indent from each line.
    lines
        .iter()
        .map(|line| {
            if line.len() >= indent {
                &line[indent..]
            } else {
                *line
            }
        })
        .collect::<Vec<&str>>()
        .join("\n")
}

/// Merges multiple comment lines into one normalized string.
/// This function assumes that each line has already been processed (markers removed and dedented).
pub fn merge_comment_lines(lines: &[&str]) -> String {
    lines
        .iter()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<&str>>()
        .join(" ")
}


#[cfg(test)]
mod tests {
    use super::*;

    // TODO: expand the tests with multiline comments and more complex scenarios

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
    fn test_dedent_comment() {
        let input = "    TODO: This is a test\n    with indentation\n    preserved.";
        let output = dedent_comment(input);
        let expected = "TODO: This is a test\nwith indentation\npreserved.";
        assert_eq!(output, expected);
    }

    #[test]
    fn test_merge_comment_lines() {
        let lines = vec!["Fix bug", "Improve error handling", "Add logging"];
        let output = merge_comment_lines(&lines);
        let expected = "Fix bug Improve error handling Add logging";
        assert_eq!(output, expected);
    }
}

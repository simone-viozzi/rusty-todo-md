//! File and directory exclusion based on glob patterns.
//!
//! This module provides functionality to filter files and directories using glob patterns,
//! supporting wildcards like `*`, `?`, and `**` for recursive matching.

use globset::Glob;
use log::info;
use std::path::{Path, PathBuf};

/// Exclusion rule type
#[derive(Debug, Clone)]
enum ExclusionKind {
    /// Matches files and directories
    Exclude,
    /// Matches directories only
    ExcludeDir,
}

/// An exclusion rule with its pattern and kind
#[derive(Debug, Clone)]
pub struct ExclusionRule {
    pattern: String,
    kind: ExclusionKind,
    glob: globset::GlobMatcher,
}

/// Build the exclusion matcher from CLI arguments
///
/// # Arguments
/// * `exclude_patterns` - Patterns for `--exclude` flag (files or directories)
/// * `exclude_dir_patterns` - Patterns for `--exclude-dir` flag (directories only)
///
/// # Returns
/// A vector of exclusion rules or an error if any pattern is invalid
pub fn build_exclusion_matcher(
    exclude_patterns: Vec<String>,
    exclude_dir_patterns: Vec<String>,
) -> Result<Vec<ExclusionRule>, String> {
    let mut rules = Vec::new();

    // Add --exclude patterns
    for pattern in exclude_patterns {
        let normalized = normalize_pattern(&pattern);
        let glob = Glob::new(&normalized)
            .map_err(|e| format!("Invalid exclude pattern '{}': {}", pattern, e))?
            .compile_matcher();
        rules.push(ExclusionRule {
            pattern: pattern.clone(),
            kind: ExclusionKind::Exclude,
            glob,
        });
    }

    // Add --exclude-dir patterns (ensure they end with /)
    for pattern in exclude_dir_patterns {
        let pattern_with_slash = if pattern.ends_with('/') {
            pattern.clone()
        } else {
            format!("{}/", pattern)
        };
        let normalized = normalize_pattern(&pattern_with_slash);
        let glob = Glob::new(&normalized)
            .map_err(|e| format!("Invalid exclude-dir pattern '{}': {}", pattern, e))?
            .compile_matcher();
        rules.push(ExclusionRule {
            pattern: pattern_with_slash, // Store pattern with trailing slash
            kind: ExclusionKind::ExcludeDir,
            glob,
        });
    }

    Ok(rules)
}

/// Normalize a glob pattern to use forward slashes (cross-platform compatibility)
fn normalize_pattern(pattern: &str) -> String {
    pattern.replace('\\', "/")
}

/// Check if a path should be excluded based on exclusion rules
///
/// # Arguments
/// * `path` - The path to check
/// * `is_dir` - Whether the path is a directory
/// * `rules` - The exclusion rules to apply
///
/// # Returns
/// `true` if the path should be excluded (last match wins), `false` otherwise
pub fn should_exclude(path: &Path, is_dir: bool, rules: &[ExclusionRule]) -> bool {
    // Try to match against both the full path and just the file/dir name components
    let path_str = path.to_str().unwrap_or("");
    let normalized_full_path = normalize_pattern(path_str);

    // Also get just the filename/dirname for simple pattern matching
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    // Get path components for relative path matching
    let components: Vec<&str> = path
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect();

    let mut excluded = false;

    for rule in rules {
        let mut matches = false;

        // Determine if this is a directory-only pattern
        let is_dir_pattern =
            rule.pattern.ends_with('/') || matches!(rule.kind, ExclusionKind::ExcludeDir);

        if is_dir_pattern {
            // This is a directory pattern - check if this is a dir OR if any parent is this dir
            if is_dir {
                // Check if the directory itself matches
                matches =
                    rule.glob.is_match(&normalized_full_path) || rule.glob.is_match(file_name);

                if !matches {
                    for i in 0..components.len() {
                        let partial_path = components[i..].join("/") + "/";
                        if rule.glob.is_match(&partial_path) {
                            matches = true;
                            break;
                        }
                    }
                }
            } else {
                // For files, check if any parent directory matches the pattern
                // e.g., if pattern is "src/" or "build" and file is "/path/build/output.rs", exclude it
                for i in 0..components.len() - 1 {
                    // -1 to exclude the filename itself
                    // Build the path up to (but not including) the filename
                    for j in (i + 1)..components.len() {
                        // Start from i+1 to get directory paths
                        let dir_path = components[i..j].join("/");
                        // Check if this directory path matches the pattern
                        if rule.glob.is_match(&dir_path)
                            || rule.glob.is_match(&(dir_path.clone() + "/"))
                        {
                            matches = true;
                            break;
                        }
                    }
                    if matches {
                        break;
                    }
                }
            }
        } else {
            // Regular file/dir pattern
            matches = rule.glob.is_match(&normalized_full_path) || rule.glob.is_match(file_name);

            if !matches {
                for i in 0..components.len() {
                    let partial_path = components[i..].join("/");
                    if rule.glob.is_match(&partial_path) {
                        matches = true;
                        break;
                    }
                }
            }
        }

        if matches {
            excluded = true; // Last match wins
        }
    }

    excluded
}

/// Filter files based on exclusion rules
///
/// # Arguments
/// * `files` - The list of files to filter
/// * `rules` - The exclusion rules to apply
///
/// # Returns
/// A filtered list of files with excluded files removed
pub fn filter_excluded_files(files: Vec<PathBuf>, rules: &[ExclusionRule]) -> Vec<PathBuf> {
    files
        .into_iter()
        .filter(|file| {
            let is_dir = file.is_dir();
            let should_exclude_file = should_exclude(file, is_dir, rules);
            if should_exclude_file {
                info!("Excluding: {:?}", file);
            }
            !should_exclude_file
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_normalize_pattern() {
        assert_eq!(normalize_pattern("foo/bar"), "foo/bar");
        assert_eq!(normalize_pattern("foo\\bar"), "foo/bar");
        assert_eq!(normalize_pattern("foo\\bar\\baz"), "foo/bar/baz");
    }

    #[test]
    fn test_build_exclusion_matcher_exclude() {
        let rules = build_exclusion_matcher(vec!["*.log".to_string()], vec![]).unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].pattern, "*.log");
    }

    #[test]
    fn test_build_exclusion_matcher_exclude_dir() {
        let rules = build_exclusion_matcher(vec![], vec!["build".to_string()]).unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].pattern, "build/");
    }

    #[test]
    fn test_build_exclusion_matcher_exclude_dir_with_slash() {
        let rules = build_exclusion_matcher(vec![], vec!["build/".to_string()]).unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].pattern, "build/");
    }

    #[test]
    fn test_build_exclusion_matcher_multiple() {
        let rules = build_exclusion_matcher(
            vec!["*.log".to_string(), "*.tmp".to_string()],
            vec!["build".to_string()],
        )
        .unwrap();
        assert_eq!(rules.len(), 3);
    }

    #[test]
    fn test_build_exclusion_matcher_invalid_pattern() {
        let result = build_exclusion_matcher(vec!["[invalid".to_string()], vec![]);
        assert!(result.is_err());
    }

    // Parametrized test using table-driven approach
    #[test]
    fn test_should_exclude_files() {
        let test_cases = vec![
            // (pattern, path, is_dir, expected_excluded)
            ("*.log", "/tmp/file.log", false, true),
            ("*.log", "/tmp/file.txt", false, false),
            ("*.log", "/tmp/test.log", false, true),
            ("file.txt", "/tmp/file.txt", false, true),
            ("file.txt", "/tmp/other.txt", false, false),
            ("src/*.rs", "/tmp/src/main.rs", false, true),
            ("**/*.log", "/tmp/a/b/c/file.log", false, true),
            ("**/*.log", "/tmp/file.txt", false, false),
        ];

        for (pattern, path, is_dir, expected) in test_cases {
            let rules = build_exclusion_matcher(vec![pattern.to_string()], vec![]).unwrap();
            let result = should_exclude(Path::new(path), is_dir, &rules);
            assert_eq!(
                result,
                expected,
                "Pattern '{}' with path '{}' (is_dir={}) should be {} but got {}",
                pattern,
                path,
                is_dir,
                if expected { "excluded" } else { "included" },
                if result { "excluded" } else { "included" }
            );
        }
    }

    #[test]
    fn test_should_exclude_directories() {
        let test_cases = vec![
            // (pattern, path, is_dir, expected_excluded)
            ("build/", "/tmp/build", true, true),
            ("build/", "/tmp/src", true, false),
            ("src/", "/tmp/src", true, true),
            ("src/", "/tmp/src/main.rs", false, true), // File under excluded dir
        ];

        for (pattern, path, is_dir, expected) in test_cases {
            let rules = build_exclusion_matcher(vec![pattern.to_string()], vec![]).unwrap();
            let result = should_exclude(Path::new(path), is_dir, &rules);
            assert_eq!(
                result,
                expected,
                "Pattern '{}' with path '{}' (is_dir={}) should be {} but got {}",
                pattern,
                path,
                is_dir,
                if expected { "excluded" } else { "included" },
                if result { "excluded" } else { "included" }
            );
        }
    }

    #[test]
    fn test_should_exclude_exclude_dir_flag() {
        let test_cases = vec![
            // (pattern, path, is_dir, expected_excluded)
            ("build", "/tmp/build", true, true),
            ("build", "/tmp/build/file.rs", false, true), // File under excluded dir
            ("build", "/tmp/src", true, false),
        ];

        for (pattern, path, is_dir, expected) in test_cases {
            let rules = build_exclusion_matcher(vec![], vec![pattern.to_string()]).unwrap();
            let result = should_exclude(Path::new(path), is_dir, &rules);
            assert_eq!(
                result,
                expected,
                "Pattern '{}' (exclude-dir) with path '{}' (is_dir={}) should be {} but got {}",
                pattern,
                path,
                is_dir,
                if expected { "excluded" } else { "included" },
                if result { "excluded" } else { "included" }
            );
        }
    }

    #[test]
    fn test_last_match_wins() {
        // Multiple patterns, last one wins
        let rules = build_exclusion_matcher(
            vec!["*.log".to_string(), "important.log".to_string()],
            vec![],
        )
        .unwrap();

        // Both patterns match, but we're using OR logic (any match excludes)
        // so the file should be excluded
        assert!(should_exclude(
            Path::new("/tmp/important.log"),
            false,
            &rules
        ));
    }

    #[test]
    fn test_filter_excluded_files() {
        let rules = build_exclusion_matcher(vec!["*.log".to_string()], vec![]).unwrap();
        let files = vec![
            PathBuf::from("/tmp/file1.txt"),
            PathBuf::from("/tmp/file2.log"),
            PathBuf::from("/tmp/file3.txt"),
        ];

        let filtered = filter_excluded_files(files, &rules);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.contains(&PathBuf::from("/tmp/file1.txt")));
        assert!(filtered.contains(&PathBuf::from("/tmp/file3.txt")));
        assert!(!filtered.contains(&PathBuf::from("/tmp/file2.log")));
    }
}

use crate::todo_extractor_internal::aggregator::CommentLine;
use crate::todo_extractor_internal::languages::common::CommentParser;
use crate::todo_extractor_internal::languages::python::PythonParser;

pub struct ShellParser;

impl CommentParser for ShellParser {
    fn parse_comments(file_content: &str) -> Vec<CommentLine> {
        PythonParser::parse_comments(file_content)
    }
}

#[cfg(test)]
mod shell_tests {
    use crate::logger;
    use crate::todo_extractor_internal::aggregator::{extract_marked_items, MarkerConfig};
    use log::LevelFilter;
    use std::path::Path;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn init_logger() {
        INIT.call_once(|| {
            env_logger::Builder::from_default_env()
                .format(logger::format_logger)
                .filter_level(LevelFilter::Debug)
                .is_test(true)
                .try_init()
                .ok();
        });
    }

    #[test]
    fn test_sh_single_comment() {
        init_logger();
        let src = r#"# TODO: do stuff
echo hello"#;
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };
        let todos = extract_marked_items(Path::new("script.sh"), src, &config);
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].message, "do stuff");
    }
}

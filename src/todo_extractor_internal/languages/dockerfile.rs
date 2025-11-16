use crate::todo_extractor_internal::aggregator::{parse_comments, CommentLine};
use crate::todo_extractor_internal::languages::common::CommentParser;
use pest_derive::Parser;
use std::marker::PhantomData;

#[derive(Parser)]
#[grammar = "todo_extractor_internal/languages/dockerfile.pest"]
pub struct DockerfileParser;

impl CommentParser for DockerfileParser {
    fn parse_comments(file_content: &str) -> Vec<CommentLine> {
        parse_comments::<Self, Rule>(PhantomData, Rule::dockerfile_file, file_content)
    }
}

#[cfg(test)]
mod dockerfile_tests {
    use crate::todo_extractor_internal::aggregator::MarkerConfig;
    use std::path::Path;

    use crate::test_utils::{init_logger, test_extract_marked_items};

    #[test]
    fn test_dockerfile_single_comment() {
        init_logger();
        let src = r#"# TODO: install packages
FROM alpine"#;
        let config = MarkerConfig {
            markers: vec!["TODO:".to_string()],
        };

        // TODO now in the tests i need to actually create the file instead of passing a fake path and a content
        let todos = test_extract_marked_items(Path::new("Dockerfile"), src, &config);

        // TODO and also need to check errors
        assert_eq!(todos.len(), 1);
        assert_eq!(todos[0].message, "install packages");
    }

    #[test]
    fn test_dockerfile_multiline_run_with_todo() {
        init_logger();
        let src = r#"FROM node:18-alpine

RUN apk add --no-cache \
     python3 \
     make \
     g++ \
     # need to keep dev dependencies because we use it as a dev container
     # TODO: in future: split into prod and dev containers
     uv sync --frozen --no-editable --dev

WORKDIR /app"#;
        let config = MarkerConfig {
            markers: vec!["TODO".to_string()],
        };

        let todos = test_extract_marked_items(Path::new("Dockerfile"), src, &config);

        assert_eq!(todos.len(), 1);
        assert_eq!(
            todos[0].message,
            "in future: split into prod and dev containers"
        );
        assert_eq!(todos[0].line_number, 8);
    }

    #[test]
    fn test_dockerfile_multiple_todos_and_markers() {
        init_logger();
        let src = r#"FROM alpine:latest

# TODO: Optimize image size
RUN apk add --no-cache \
    curl \
    # FIXME: Remove this dependency when possible
    wget \
    # HACK: Temporary workaround for SSL issues
    ca-certificates

WORKDIR /app

# TODO: Add health check
COPY . .

# Regular comment without markers
EXPOSE 8080

# FIXME: Use proper user instead of root
USER root

CMD ["./app"]"#;
        let config = MarkerConfig {
            markers: vec!["TODO".to_string(), "FIXME".to_string(), "HACK".to_string()],
        };

        let todos = test_extract_marked_items(Path::new("Dockerfile"), src, &config);

        assert_eq!(todos.len(), 5); // Updated to match actual count
                                    // Sort todos by line number to make assertions more predictable
        let mut sorted_todos = todos.clone();
        sorted_todos.sort_by_key(|t| t.line_number);

        assert_eq!(sorted_todos[0].marker, "TODO");
        assert_eq!(sorted_todos[0].message, "Optimize image size");
        assert_eq!(sorted_todos[0].line_number, 3);

        assert_eq!(sorted_todos[1].marker, "FIXME");
        assert_eq!(
            sorted_todos[1].message,
            "Remove this dependency when possible"
        );
        assert_eq!(sorted_todos[1].line_number, 6);

        assert_eq!(sorted_todos[2].marker, "HACK");
        assert_eq!(
            sorted_todos[2].message,
            "Temporary workaround for SSL issues"
        );
        assert_eq!(sorted_todos[2].line_number, 8);

        assert_eq!(sorted_todos[3].marker, "TODO");
        assert_eq!(sorted_todos[3].message, "Add health check");
        assert_eq!(sorted_todos[3].line_number, 13);

        assert_eq!(sorted_todos[4].marker, "FIXME");
        assert_eq!(sorted_todos[4].message, "Use proper user instead of root");
        assert_eq!(sorted_todos[4].line_number, 19);
    }

    #[test]
    fn test_dockerfile_mixed_comment_styles() {
        init_logger();
        let src = r#"# TODO: Base image selection
FROM node:24-alpine

RUN apt-get update && \
    # FIXME: Pin package versions
    apt-get install -y python3 && \
    apt-get clean

# Regular comment
WORKDIR /app

COPY package*.json ./

# TODO Install dependencies and build
RUN npm install && \
    npm run build

EXPOSE 3000"#;
        let config = MarkerConfig {
            markers: vec!["TODO".to_string(), "FIXME".to_string()],
        };

        let todos = test_extract_marked_items(Path::new("Dockerfile"), src, &config);

        assert_eq!(todos.len(), 3);
        // Sort todos by line number to make assertions more predictable
        let mut sorted_todos = todos.clone();
        sorted_todos.sort_by_key(|t| t.line_number);

        assert_eq!(sorted_todos[0].message, "Base image selection");
        assert_eq!(sorted_todos[0].line_number, 1);

        assert_eq!(sorted_todos[1].message, "Pin package versions");
        assert_eq!(sorted_todos[1].line_number, 5);

        assert_eq!(sorted_todos[2].message, "Install dependencies and build");
        assert_eq!(sorted_todos[2].line_number, 14);
    }
}

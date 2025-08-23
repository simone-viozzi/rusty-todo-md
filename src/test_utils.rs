use crate::{
    logger,
    todo_extractor_internal::aggregator::{
        extract_marked_items_with_parser, get_effective_extension, get_parser_for_extension,
    },
    MarkedItem, MarkerConfig,
};
use log::LevelFilter;
use std::{path::Path, sync::Once};

static INIT: Once = Once::new();

pub(crate) fn init_logger() {
    INIT.call_once(|| {
        env_logger::Builder::from_default_env()
            .format(logger::format_logger)
            .filter_level(LevelFilter::Debug)
            .is_test(true)
            .try_init()
            .ok();
    });
}

pub(crate) fn test_extract_marked_items(
    file: &Path,
    src: &str,
    marker_config: &MarkerConfig,
) -> Vec<MarkedItem> {
    let effective_ext = get_effective_extension(file);
    let parser_fn = match get_parser_for_extension(&effective_ext, file) {
        Some(parser) => parser,
        None => {
            // Skip unsupported file types without reading content
            return Vec::new();
        }
    };

    extract_marked_items_with_parser(file, src, parser_fn, marker_config)
}

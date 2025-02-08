//pub mod ts;
pub mod python;
pub mod rust;
//pub mod js;
//pub mod go;

//pub use ts::parse_ts_comments;
pub use python::parse_python_comments;
pub use rust::parse_rust_comments;
//pub use js::parse_js_comments;
//pub use go::parse_go_comments;

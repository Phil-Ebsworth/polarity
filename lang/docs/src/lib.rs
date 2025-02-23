pub mod doc;
pub mod generate;
pub mod generate_docs;
pub mod util;
pub mod structure;

pub use doc::write_html;
pub use util::open;
pub use structure::generate_html_from_paths;

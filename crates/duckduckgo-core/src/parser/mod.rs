mod block;
mod decode;
mod pagination;
mod results;

pub use block::{BlockReason, classify_block};
pub use results::{ParsedPage, parse_html};

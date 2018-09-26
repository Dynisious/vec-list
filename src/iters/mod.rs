
mod iter;
mod iter_mut;
mod drain;
mod drain_filter;

pub use self::{
    iter::*,
    iter_mut::*,
    drain::*,
    drain_filter::*,
};

#[macro_export]
macro_rules! node_err {
    () => (format!("VecList nodes not maintained: at {} {}:{}", file!(), line!(), column!(),))
}

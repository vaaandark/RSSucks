#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub mod feed;
pub mod opml;
mod article;
pub use app::RSSucks;

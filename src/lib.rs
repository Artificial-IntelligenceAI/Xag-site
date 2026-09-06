//! What the site is made of, apart from the browser.
//!
//! The prose, the small markup it is written in, the tokeniser that colours
//! Xag, and the addresses — none of which needs a page open to be true, and all
//! of which is read by two things: the app, and the generator that writes the
//! HTML served before the app has loaded.

pub mod content;
pub mod html;
pub mod markup;
pub mod route;
pub mod syntax;

//! Example library of [`cargo-sync-rdme`]
//!
//! This is document comments embedded in the source code.
//! It will be extracted and used to generate README.md.
//!
//! # Intra-doc link support
//!
//! Intra-doc links are also supported.
//!
//! ## Source code
//!
//! ```markdown
//! - [`crate::add`]
//! - [add1](crate::add)
//! - [add2](add)
//! - [add3]
//!
//! [add3]: add
//! ```
//!
//! ## Rendered
//!
//! - [`crate::add`]
//! - [add1](crate::add)
//! - [add2](add)
//! - [add3]
//!
//! [add3]: add

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

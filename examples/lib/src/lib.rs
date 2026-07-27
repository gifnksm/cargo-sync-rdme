//! Example library of `cargo-sync-rdme`.
//!
//! This is document comments embedded in the source code.
//! It will be extracted and used to generate README.md.
//!
//! # Intra-doc link support
//!
//! Intra-doc links are also supported.
//!
//! ## Supported Syntax
//!
//! [All rustdoc syntax for intra-doc links][intra-doc-link] is supported.
//!
//! [intra-doc-link]: https://doc.rust-lang.org/rustdoc/write-documentation/linking-to-items-by-name.html
//!
//! ### Source code
//!
//! ```markdown
//! * Normal link: [the struct](Struct)
//! * Normal with backtick link: [the struct](`Struct`)
//! * Reference link: [the enum][e1]
//! * Reference link with backtick: [the enum][e2]
//! * Reference shortcut link: [Union]
//! * Reference shortcut link with backtick: [`Union`]
//!
//! * Link with paths: [`crate::Struct`], [`self::Struct`]
//! * Link with namespace: [`Struct`](struct@Struct), [`macro_`](macro@macro_)
//! * Link with disambiguators: [`function()`], [`macro_!`]
//!
//! [e1]: Enum
//! [e2]: `Enum`
//! ```
//!
//! ### Rendered
//!
//! * Normal link: [the struct](Struct)
//! * Normal with backtick link: [the struct](`Struct`)
//! * Reference link: [the enum][e1]
//! * Reference link with backtick: [the enum][e2]
//! * Reference shortcut link: [Union]
//! * Reference shortcut link with backtick: [`Union`]
//!
//! * Link with paths: [`crate::Struct`], [`self::Struct`]
//! * Link with namespace: [`Struct`](struct@Struct), [`macro_`](macro@macro_)
//! * Link with disambiguators: [`function()`], [`macro_!`]
//!
//! [e1]: Enum
//! [e2]: `Enum`
//!
//! ## Link showcase
//!
//! <!-- markdownlint-disable MD060 -->
//! | Item Kind           | [`crate`]          | [`std`]                       | External Crate                               |
//! | --------------------| ------------------ | ----------------------------- | -------------------------------------------- |
//! | Module              | [`module`]         | [`std::collections`]          | [`num::bigint`]                              |
//! | Struct              | [`Struct`]         | [`std::collections::HashMap`] | [`num::bigint::BigInt`]                      |
//! | Struct Field        | [`Struct::field`]  | [`std::ops::Range::start`]    |                                              |
//! | Union               | [`Union`]          |                               |                                              |
//! | Enum                | [`Enum`]           | [`Option`]                    | [`num::traits::FloatErrorKind`]              |
//! | Enum Variant        | [`Enum::Variant`]  | [`Option::Some`]              | [`num::traits::FloatErrorKind::Empty`]       |
//! | Function            | [`function`]       | [`std::iter::from_fn`]        | [`num::abs`]                                 |
//! | Typedef             | [`Typedef`]        | [`std::io::Result`]           | [`num::BigRational`]                         |
//! | Constant            | [`CONSTANT`]       | [`std::path::MAIN_SEPARATOR`] |                                              |
//! | Trait               | [`Trait`]          | [`std::clone::Clone`]         | [`num::Num`]                                 |
//! | Method (trait)      | [`Trait::method`]  | [`std::clone::Clone::clone`]  | [`num::Num::from_str_radix`]                 |
//! | Method (impl)       | [`Struct::method`] | [`Vec::clone`]                | [`num::bigint::BigInt::from_str_radix`]      |
//! | Static              | [`STATIC`]         |                               |                                              |
//! | Macro               | [`macro_`]         | [`println`]                   |                                              |
//! | Attribute Macro     |                    |                               | [`async_trait::async_trait`]                 |
//! | Derive Macro        |                    |                               | [`serde::Serialize`](macro@serde::Serialize) |
//! | Associated Constant | [`Trait::CONST`]   | [`i32::MAX`]                  |                                              |
//! | Associated Type     | [`Trait::Type`]    | [`Iterator::Item`]            |                                              |
//! | Primitive           |                    | [`i32`]                       |                                              |
//! <!-- markdownlint-enable MD060 -->
//!
//! ### Code Block
//!
//! Fenced code block:
//!
//! ```
//! # fn main() {
//! println!("Hello, world!");
//! # }
//! ```
//!
//! Indented code block:
//!
//!     # fn main() {
//!     println!("Hello, world!");
//!     # }

#![allow(missing_copy_implementations, missing_debug_implementations)]

// import unused external crates to demonstrate intra-doc links to external crates
use async_trait as _;
use num as _;
use serde as _;

#[cfg(doc)]
use num::Num as _;

/// This is a module.
pub mod module {}

/// This is a struct.
pub struct Struct {
    /// This is a struct field.
    pub field: usize,
}

/// This is union.
pub union Union {
    /// This is a first union field.
    pub x: u32,
    /// This is a second union field.
    pub y: i32,
}

/// This is an enum.
pub enum Enum {
    /// This is an enum variant.
    Variant,
}

/// This is a function.
pub fn function() {}

/// This is a type definition.
pub type Typedef = i32;

/// This is a constant.
pub const CONSTANT: &str = "This is a constant.";

/// This is a trait.
pub trait Trait {
    /// This is a trait method.
    fn method(&self);

    /// This is an associated constant.
    const CONST: &'static str;

    /// This is an associated type.
    type Type: Trait;
}

/// This is an impl.
impl Trait for Struct {
    fn method(&self) {}

    const CONST: &'static str = "This is an associated constant.";

    type Type = Struct;
}

/// This is a static.
pub static STATIC: &str = "This is a static.";

/// This is a macro.
#[macro_export]
macro_rules! macro_ {
    () => {};
}

//! Example library of `cargo-sync-rdme`.
//!
//! This is document comments embedded in the source code.
//! It will be extracted and used to generate README.md.
//!
//! # Intra-doc Links
//!
//! [All intra-doc link syntaxes][intra-doc-link] are supported.
//!
//! [intra-doc-link]: https://doc.rust-lang.org/rustdoc/write-documentation/linking-to-items-by-name.html
//!
//! ## Markdown Link Syntaxes
//!
//! * Inline links:
//!   * `[the struct](Struct)`
//!     → [the struct](Struct)
//!   * ``[the struct](`Struct`)``
//!     → [the struct](`Struct`)
//! * Full reference links:
//!   * `[the struct][Struct]`
//!     → [the struct][Struct]
//!   * ``[the struct][`Struct`]``
//!     → [the struct][`Struct`]
//!   * `[the struct][struct-without-backtick]`
//!     → [the struct][struct-without-backtick]
//!   * `[the struct][struct-with-backtick]`
//!     → [the struct][struct-with-backtick]
//! * Collapsed reference links:
//!   * `[struct-without-backtick][]`
//!     → [struct-without-backtick][]
//!   * `[struct-with-backtick][]`
//!     → [struct-with-backtick][]
//!   * `[Struct][]`
//!     → [Struct][]
//!   * ``[`Struct`][]``
//!     → [`Struct`][]
//! * Shortcut reference links:
//!   * `[struct-without-backtick]`
//!     → [struct-without-backtick]
//!   * `[struct-with-backtick]`
//!     → [struct-with-backtick]
//!   * `[Struct]`
//!     → [Struct]
//!   * ``[`Struct`]``
//!     → [`Struct`]
//!
//! ## Intra-doc Link Syntaxes
//!
//! * Links with paths:
//!   * ``[`crate::Struct`]``
//!     → [`crate::Struct`]
//!   * ``[`self::Struct`]``
//!     → [`self::Struct`]
//! * Links with namespaces:
//!   * ``[`struct@Struct`]``
//!     → [`struct@Struct`]
//!   * ``[`enum@Enum`]``
//!     → [`enum@Enum`]
//!   * ``[`trait@Trait`]``
//!     → [`trait@Trait`]
//!   * ``[`union@Union`]``
//!     → [`union@Union`]
//!   * ``[`mod@module`], [`module@module`]``
//!     → [`mod@module`], [`module@module`]
//!   * ``[`const@CONSTANT`], [`constant@CONSTANT`]``
//!     → [`const@CONSTANT`], [`constant@CONSTANT`]
//!   * ``[`fn@function`], [`function@function`]``
//!     → [`fn@function`], [`function@function`]
//!   * ``[`field@Struct::field`]``
//!     → [`field@Struct::field`]
//!   * ``[`variant@Enum::Variant`]``
//!     → [`variant@Enum::Variant`]
//!   * ``[`method@Trait::method`]``
//!     → [`method@Trait::method`]
//!   * ``[`derive@Clone`]``
//!     → [`derive@Clone`]
//!   * ``[`type@Struct`]``
//!     → [`type@Struct`]
//!   * ``[`value@STATIC`]``
//!     → [`value@STATIC`]
//!   * ``[`macro@declarative_macro`]`` → [`macro@declarative_macro`]
//!   * ``[`tyalias@TypeAlias`], [`typealias@TypeAlias`]``
//!     → [`tyalias@TypeAlias`], [`typealias@TypeAlias`]
//!   * ``[`prim@i32`], [`primitive@i32`]``
//!     → [`prim@i32`], [`primitive@i32`]
//! * Links with disambiguators:
//!   * ``[`function()`]``
//!     → [`function()`]
//!   * ``[`declarative_macro!`]``
//!     → [`declarative_macro!`]
//!   * ``[`declarative_macro!()`]``
//!     → [`declarative_macro!()`]
//!   * ``[`declarative_macro![]`](declarative_macro![])``
//!     → [`declarative_macro![]`](declarative_macro![])
//!   * ``[`declarative_macro!{}`]``
//!     → [`declarative_macro!{}`]
//!
//! [struct-without-backtick]: Struct
//! [struct-with-backtick]: `Struct`
//!
//! ## Link showcase
//!
//! <!-- markdownlint-disable MD060 -->
//! | Link Target                                     | [`crate`]                     | [`std`]                         | External Crate                         |
//! | ----------------------------------------------- | ----------------------------- | ------------------------------- | -------------------------------------- |
//! | Module                                          | [`module`]                    | [`std::collections`]            | [`num::bigint`]                        |
//! | Struct                                          | [`Struct`]                    | [`std::collections::HashMap`]   | [`num::BigInt`]                        |
//! | Struct Field                                    | [`Struct::field`]             | [`std::ops::Range::start`]      | [`num::Complex::re`]                   |
//! | Tuple Struct Field                              | [`TupleStruct::0`]            | [`std::cmp::Reverse::0`]        |                                        |
//! | Union                                           | [`Union`]                     | [`std::mem::MaybeUninit`]       |                                        |
//! | Union Field                                     | [`Union::field1`]             |                                 |                                        |
//! | Enum                                            | [`Enum`]                      | [`Option`]                      | [`num::traits::FloatErrorKind`]        |
//! | Enum Variant                                    | [`Enum::Variant`]             | [`Option::Some`]                | [`num::traits::FloatErrorKind::Empty`] |
//! | Variant Field                                   | [`Enum::Struct::field`]       |                                 |                                        |
//! | Tuple Variant Field                             | [`Enum::Tuple::0`]            | [`Option::Some::0`]             | [`serde::de::Unexpected::Other::0`]    |
//! | Type Alias                                      | [`TypeAlias`]                 | [`std::fmt::Result`]            | [`num::BigRational`]                   |
//! | Trait                                           | [`Trait`]                     | [`Iterator`]                    | [`num::Num`]                           |
//! | Required Method                                 | [`Trait::method`]             | [`Iterator::next`]              | [`num::Zero::is_zero`]                 |
//! | Provided Method                                 | [`Trait::provided_method`]    | [`Iterator::size_hint`]         | [`num::Zero::set_zero`]                |
//! | Required Associated Function                    | [`Trait::assoc_fn`]           | [`FromIterator::from_iter`]     | [`num::Zero::zero`]                    |
//! | Provided Associated Function                    | [`Trait::provided_assoc_fn`]  | [`std::iter::Step::forward`]    | [`num::FromPrimitive::from_i32`]       |
//! | Required Associated Constant                    | [`Trait::CONST`]              |                                 | [`num::traits::ConstZero::ZERO`]       |
//! | Required Associated Type                        | [`Trait::Type`]               | [`Iterator::Item`]              | [`num::Num::FromStrRadixErr`]          |
//! | Trait Implementation Method                     | [`Struct::method`]            | [`Vec::clone`]                  | [`num::BigInt::is_zero`]               |
//! | Trait Implementation Method (overrides default) | [`Struct::provided_method`]   | [`std::slice::Iter::size_hint`] | [`num::BigInt::set_zero`]              |
//! | Trait Implementation Associated Function        | [`Struct::assoc_fn`]          | [`Vec::from_iter`]              | [`num::BigInt::zero`]                  |
//! | Trait Implementation Associated Constant        | [`Struct::CONST`]             |                                 |                                        |
//! | Trait Implementation Associated Type            | [`Struct::Type`]              | [`std::slice::Iter::Item`]      | [`num::BigInt::FromStrRadixErr`]       |
//! | Inherent Method                                 | [`Struct::inhr_method`]       | [`Vec::len`]                    | [`num::BigInt::sign`]                  |
//! | Inherent Associated Function                    | [`Struct::inhr_assoc_fn`]     | [`Vec::new`]                    | [`num::BigInt::new`]                   |
//! | Inherent Associated Constant                    | [`Struct::INHR_CONST`]        | [`std::time::Duration::ZERO`]   | [`num::BigInt::ZERO`]                  |
//! | Constant                                        | [`CONSTANT`]                  | [`std::path::MAIN_SEPARATOR`]   |                                        |
//! | Static                                          | [`STATIC`]                    |                                 |                                        |
//! | Function                                        | [`function`]                  | [`std::iter::from_fn`]          | [`num::abs`]                           |
//! | Primitive Type                                  |                               | [`i32`]                         |                                        |
//! | Primitive Method                                |                               | [`i32::count_ones`]             |                                        |
//! | Primitive Associated Function                   |                               | [`i32::from_str_radix`]         |                                        |
//! | Primitive Associated Constant                   |                               | [`i32::MAX`]                    |                                        |
//! | Declarative Macro                               | [`declarative_macro`]         | [`println`]                     |                                        |
//! | Attribute Macro                                 |                               | [`derive`]                      | [`async_trait::async_trait`]           |
//! | Derive Macro                                    |                               | [`derive@Clone`]                | [`derive@serde::Serialize`]            |
//! | Re-exported from Private Module                 | [`ReexportedFromPrivateMod`]  |                                 |                                        |
//! | Foreign Function                                | [`foreign_function`]          |                                 |                                        |
//! | Foreign Static                                  | [`FOREIGN_STATIC`]            |                                 |                                        |
//! <!-- markdownlint-enable MD060 -->
//!
//! * crate without `html_root_url`: [`cargo_metadata::MetadataCommand`]
//!
//! # Code Blocks
//!
//! All code block syntaxes in [CommonMark Spec][commonmark-spec] are supported.
//!
//! In rendered Rust code blocks, `cargo-sync-rdme` matches the hidden-line handling of `rustdoc` for `#`-prefixed lines.
//!
//! [commonmark-spec]: https://spec.commonmark.org/0.31.2/
//!
//! ## Fenced code block
//!
//! **Source:**
//!
//! ````markdown
//! ```
//! # fn main() {
//! println!("Hello, world!");
//! # }
//! ```
//! ````
//!
//! **Rendered:**
//!
//! ```
//! # fn main() {
//! println!("Hello, world!");
//! # }
//! ```
//!
//! ## Indented code block
//!
//! **Source:**
//!
//! ```markdown
//!     # fn main() {
//!     println!("Hello, world!");
//!     # }
//! ```
//!
//! **Rendered:**
//!
//!     # fn main() {
//!     println!("Hello, world!");
//!     # }
//!
//! # `rustdoc` Markdown Extensions
//!
//! `cargo-sync-rdme` preserves several Markdown extensions supported by `rustdoc`.
//!
//! <!-- markdownlint-disable MD060 -->
//! | Extension | Example |
//! | --------- | ------- |
//! | Tables | This section itself starts with a table. |
//! | Footnotes | Footnotes work in prose too.[^markdown-extension-footnote] |
//! | Strikethrough | ~~Deprecated wording~~ |
//! | Task lists | See the checklist below. |
//! <!-- markdownlint-enable MD060 -->
//!
//! - [x] Completed task list item
//! - [ ] Incomplete task list item
//!
//! `rustdoc` also applies smart punctuation, and `cargo-sync-rdme` preserves
//! those conversions in synced Markdown so README output matches `rustdoc`
//! more closely.
//!
//! "quoted text"... really -- exactly --- like this.
//!
//! In `rustdoc`, that text is rendered with typographic quotes, an ellipsis,
//! and en/em dashes.
//!
//! [^markdown-extension-footnote]: In `rustdoc`, footnotes are collected at the end of the rendered Markdown block.
//!

#![allow(missing_copy_implementations, missing_debug_implementations)]

// import unused external crates to demonstrate intra-doc links to external crates
use async_trait as _;
use cargo_metadata as _;
use num as _;
use serde as _;

#[cfg(doc)]
use num::{Num as _, traits::Zero as _};

/// This is a module.
pub mod module {}

/// This is a struct.
pub struct Struct {
    /// This is a struct field.
    pub field: usize,
}

/// This is a tuple struct.
pub struct TupleStruct(
    /// This is a field in a tuple struct.
    pub usize,
);

/// This is a union.
pub union Union {
    /// This is the first union field.
    pub field1: u32,
    /// This is the second union field.
    pub field2: i32,
}

/// This is an enum.
pub enum Enum {
    /// This is an enum variant.
    Variant,
    /// This is an enum variant with a field.
    Struct {
        /// This is a field in an enum struct variant.
        field: usize,
    },
    /// This is an enum tuple variant.
    Tuple(
        /// This is a field in an enum tuple variant.
        usize,
    ),
}

/// This is a function.
pub fn function() {}

/// This is a type alias.
pub type TypeAlias = i32;

/// This is a constant.
pub const CONSTANT: &str = "This is a constant.";

/// This is a trait.
pub trait Trait {
    /// This is a required method.
    fn method(&self);

    /// This is a provided method.
    fn provided_method(&self) {}

    /// This is a required associated function.
    fn assoc_fn();

    /// This is a provided associated function.
    fn provided_assoc_fn() {}

    /// This is a required associated constant.
    const CONST: &'static str;

    /// This is a required associated type.
    type Type: Trait;

    // unstable feature `associated_type_defaults` is not yet available in stable Rust, so this part is commented out
    // /// This is a trait associated type with a default type.
    // type DefaultType: Trait = Struct;
}

/// This is a trait implementation.
impl Trait for Struct {
    /// This is an implementation of the method.
    fn method(&self) {}

    /// This is an implementation of the provided method.
    fn provided_method(&self) {}

    /// This is an implementation of the associated function.
    fn assoc_fn() {}

    /// This is an implementation of the associated constant.
    const CONST: &'static str = "This is an associated constant.";

    /// This is an implementation of the associated type.
    type Type = Struct;
}

/// This is an inherent implementation.
impl Struct {
    /// This is an associated function.
    pub fn inhr_assoc_fn() {}

    /// This is a method.
    pub fn inhr_method(&self) {}

    /// This is an inherent associated constant.
    pub const INHR_CONST: &'static str = "This is an inherent associated constant.";

    // unstable feature `inherent_associated_types` is not yet available in stable Rust, so this part is commented out
    // /// This is an inherent associated type.
    // pub type InhrType = Struct;
}

/// This is a static.
pub static STATIC: &str = "This is a static.";

/// This is a declarative macro.
#[macro_export]
macro_rules! declarative_macro {
    () => {};
}

/// This is a public re-export of a struct from a private module.
pub use self::private::ReexportedFromPrivateMod;

mod private {
    /// This is a struct defined in a private module and re-exported publicly.
    pub struct ReexportedFromPrivateMod;
}

unsafe extern "C" {
    /// This is a foreign function.
    pub unsafe fn foreign_function();
    /// This is a foreign static.
    pub unsafe static FOREIGN_STATIC: i32;
}

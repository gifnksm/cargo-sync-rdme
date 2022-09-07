//! Example library of [`cargo-sync-rdme`]
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
//! - [Struct]
//! - [`Struct`]
//! - [the union](Union)
//! - [the union](`Union`)
//! - [the enum][e]
//!
//! [e]: Enum
//! ```
//!
//! ### Rendered
//!
//! - [Struct]
//! - [`Struct`]
//! - [the union](Union)
//! - [the union](`Union`)
//! - [the enum][e]
//!
//! [e]: Enum
//!
//! ## Link showcase
//!
//! | Item Kind                | [`crate`]          | [`std`]                       | External Crate                          |
//! | ------------------------ | ------------------ | ----------------------------- | --------------------------------------- |
//! | Module                   | [`module`]         | [`std::collections`]          | [`num::bigint`]                         |
//! | Struct                   | [`Struct`]         | [`std::collections::HashMap`] | [`num::bigint::BigInt`]                 |
//! | Struct Field [^1]        | [`Struct::field`]  | [`std::ops::Range::start`]    |                                         |
//! | Union                    | [`Union`]          |                               |                                         |
//! | Enum                     | [`Enum`]           | [`Option`]                    | [`num::traits::FloatErrorKind`]         |
//! | Enum Variant [^2]        | [`Enum::Variant`]  | [`Option::Some`]              | [`num::traits::FloatErrorKind::Empty`]  |
//! | Function                 | [`function`]       | [`std::iter::from_fn`]        | [`num::abs`]                            |
//! | Typedef                  | [`Typedef`]        | [`std::io::Result`]           | [`num::BigRational`]                    |
//! | Constant                 | [`CONSTANT`]       | [`std::path::MAIN_SEPARATOR`] |                                         |
//! | Trait                    | [`Trait`]          | [`std::clone::Clone`]         | [`num::Num`]                            |
//! | Method (trait) [^3]      | [`Trait::method`]  | [`std::clone::Clone::clone`]  | [`num::Num::from_str_radix`]            |
//! | Method (impl) [^3]       | [`Struct::method`] | [`Vec::clone`]                | [`num::bigint::BigInt::from_str_radix`] |
//! | Static                   | [`STATIC`]         |                               |                                         |
//! | Macro                    | [`macro_`]         | [`println`]                   |                                         |
//! | Attribute Macro          |                    |                               | [`async_trait::async_trait`]            |
//! | Derive Macro             |                    |                               | [`macro@serde::Serialize`]              |
//! | Associated Constant [^4] | [`Trait::CONST`]   | [`i32::MAX`]                  |                                         |
//! | Associated Type [^4]     | [`Trait::Type`]    | [`Iterator::Item`]            |                                         |
//! | Primitive                |                    | [`i32`]                       |                                         |
//!
//! [^1]: Intra-doc links to struct fields are not supported in cargo-sync-rdme yet due to [rustdoc bug].
//!
//! [^2]: Intra-doc links to enum variants are not supported in cargo-sync-rdme yet due to [rustdoc bug].
//!
//! [^3]: Intra-doc links to methods are not supported in cargo-sync-rdme yet due to [rustdoc bug].
//!
//! [^4]: Intra-doc links to associated constants or associated types are not supported in cargo-sync-rdme yet due to [rustdoc bug].
//!
//! [rustdoc bug]: https://github.com/rust-lang/rust/issues/101531

pub use module::add as add1;
#[cfg(doc)]
use num::Num as _;

/// This is a module.
pub mod module {
    use num::bigint::BigInt;

    pub fn add(left: usize, right: usize) -> usize {
        left + right
    }

    pub fn add_bigint(left: &BigInt, right: &BigInt) -> BigInt {
        left + right
    }
}

/// This is a struct.
pub struct Struct {
    /// This is a struct field.
    pub field: usize,
}

/// This is union.
pub union Union {
    pub x: u32,
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

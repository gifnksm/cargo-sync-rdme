//! This is an external crate.

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

    /// This is a provided associated function.
    fn provided_assoc_fn() {}

    /// This is a required associated function.
    fn assoc_fn();

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

<!-- cargo-sync-rdme title [[ -->
# cargo-sync-rdme-example-lib
<!-- cargo-sync-rdme ]] -->
<!-- cargo-sync-rdme badge -->
<!-- cargo-sync-rdme rustdoc [[ -->
Example library of `cargo-sync-rdme`.

This is document comments embedded in the source code.
It will be extracted and used to generate README.md.

## Intra-doc Links

[All intra-doc link syntaxes][intra-doc-link] are supported.

### Markdown Link Syntaxes

* Inline links:
  * `[the struct](Struct)`
    → [the struct][Struct]
  * ``[the struct](`Struct`)``
    → [the struct][`Struct`]
* Full reference links:
  * `[the struct][Struct]`
    → [the struct][Struct]
  * ``[the struct][`Struct`]``
    → [the struct][`Struct`]
  * `[the struct][struct-without-backtick]`
    → [the struct][struct-without-backtick]
  * `[the struct][struct-with-backtick]`
    → [the struct][struct-with-backtick]
* Collapsed reference links:
  * `[struct-without-backtick][]`
    → [struct-without-backtick][]
  * `[struct-with-backtick][]`
    → [struct-with-backtick][]
  * `[Struct][]`
    → [Struct][]
  * ``[`Struct`][]``
    → [`Struct`][]
* Shortcut reference links:
  * `[struct-without-backtick]`
    → [struct-without-backtick]
  * `[struct-with-backtick]`
    → [struct-with-backtick]
  * `[Struct]`
    → [Struct]
  * ``[`Struct`]``
    → [`Struct`]

### Intra-doc Link Syntaxes

* Links with paths:
  * ``[`crate::Struct`]``
    → [`crate::Struct`]
  * ``[`self::Struct`]``
    → [`self::Struct`]
* Links with namespaces:
  * ``[`struct@Struct`]``
    → [`Struct`]
  * ``[`enum@Enum`]``
    → [`Enum`]
  * ``[`trait@Trait`]``
    → [`Trait`]
  * ``[`union@Union`]``
    → [`Union`]
  * ``[`mod@module`], [`module@module`]``
    → [`module`], [`module`]
  * ``[`const@CONSTANT`], [`constant@CONSTANT`]``
    → [`CONSTANT`], [`CONSTANT`]
  * ``[`fn@function`], [`function@function`]``
    → [`function`], [`function`]
  * ``[`field@Struct::field`]``
    → [`Struct::field`]
  * ``[`variant@Enum::Variant`]``
    → [`Enum::Variant`]
  * ``[`method@Trait::method`]``
    → [`Trait::method`]
  * ``[`derive@Clone`]``
    → [`Clone`]
  * ``[`type@Struct`]``
    → [`Struct`]
  * ``[`value@STATIC`]``
    → [`STATIC`]
  * ``[`macro@declarative_macro`]`` → [`declarative_macro`]
  * ``[`tyalias@TypeAlias`], [`typealias@TypeAlias`]``
    → [`TypeAlias`], [`TypeAlias`]
  * ``[`prim@i32`], [`primitive@i32`]``
    → [`i32`], [`i32`]
* Links with disambiguators:
  * ``[`function()`]``
    → [`function()`]
  * ``[`declarative_macro!`]``
    → [`declarative_macro!`]
  * ``[`declarative_macro!()`]``
    → [`declarative_macro!()`]
  * ``[`declarative_macro![]`](declarative_macro![])``
    → [`declarative_macro![]`][declarative_macro!\[\]]
  * ``[`declarative_macro!{}`]``
    → [`declarative_macro!{}`]

### Link showcase

<!-- markdownlint-disable MD060 -->

|Link Target|[`crate`]|[`std`]|External Crate|
|-----------|-------|-----|--------------|
|Module|[`module`]|[`std::collections`]|[`num::bigint`]|
|Struct|[`Struct`]|[`std::collections::HashMap`]|[`num::BigInt`][num::BigInt@1]|
|Struct Field|[`Struct::field`]|[`std::range::Range::start`]|[`num::Complex::re`]|
|Tuple Struct Field|[`TupleStruct::0`]|[`std::cmp::Reverse::0`]||
|Union|[`Union`]|[`std::mem::MaybeUninit`]||
|Union Field|[`Union::field1`]|||
|Enum|[`Enum`]|[`Option`]|[`num::traits::FloatErrorKind`]|
|Enum Variant|[`Enum::Variant`]|[`Option::Some`]|[`num::traits::FloatErrorKind::Empty`]|
|Variant Field|[`Enum::Struct::field`]|||
|Tuple Variant Field|[`Enum::Tuple::0`]|[`Option::Some::0`]|[`serde::de::Unexpected::Other::0`]|
|Type Alias|[`TypeAlias`]|[`std::fmt::Result`]|[`num::BigRational`]|
|Trait|[`Trait`]|[`Iterator`]|[`num::Num`]|
|Required Method|[`Trait::method`]|[`Iterator::next`]|[`num::Zero::is_zero`]|
|Provided Method|[`Trait::provided_method`]|[`Iterator::size_hint`]|[`num::Zero::set_zero`]|
|Required Associated Function|[`Trait::assoc_fn`]|[`FromIterator::from_iter`]|[`num::Zero::zero`]|
|Provided Associated Function|[`Trait::provided_assoc_fn`]|[`std::iter::Step::forward`]|[`num::FromPrimitive::from_i32`]|
|Required Associated Constant|[`Trait::CONST`]||[`num::traits::ConstZero::ZERO`]|
|Required Associated Type|[`Trait::Type`]|[`Iterator::Item`]|[`num::Num::FromStrRadixErr`]|
|Trait Implementation Method|[`Struct::method`]|[`Vec::clone`]|[`num::BigInt::is_zero`]|
|Trait Implementation Method (overrides default)|[`Struct::provided_method`]|[`std::slice::Iter::size_hint`]|[`num::BigInt::set_zero`]|
|Trait Implementation Associated Function|[`Struct::assoc_fn`]|[`Vec::from_iter`]|[`num::BigInt::zero`]|
|Trait Implementation Associated Constant|[`Struct::CONST`]|||
|Trait Implementation Associated Type|[`Struct::Type`]|[`std::slice::Iter::Item`]|[`num::BigInt::FromStrRadixErr`]|
|Inherent Method|[`Struct::inhr_method`]|[`Vec::len`]|[`num::BigInt::sign`]|
|Inherent Associated Function|[`Struct::inhr_assoc_fn`]|[`Vec::new`]|[`num::BigInt::new`]|
|Inherent Associated Constant|[`Struct::INHR_CONST`]|[`std::time::Duration::ZERO`]|[`num::BigInt::ZERO`][num::BigInt::ZERO@1]|
|Constant|[`CONSTANT`]|[`std::path::MAIN_SEPARATOR`]||
|Static|[`STATIC`]|||
|Function|[`function`]|[`std::iter::from_fn`]|[`num::abs`]|
|Primitive Type||[`i32`]||
|Primitive Method||[`i32::count_ones`]||
|Primitive Associated Function||[`i32::from_str_radix`]||
|Primitive Associated Constant||[`i32::MAX`]||
|Declarative Macro|[`declarative_macro`]|[`println`]||
|Attribute Macro||[`derive`]|[`async_trait::async_trait`]|
|Derive Macro||[`Clone`]|[`serde::Serialize`]|
|Re-exported from Private Module|[`ReexportedFromPrivateMod`]|||
|Foreign Function|[`foreign_function`]|||
|Foreign Static|[`FOREIGN_STATIC`]|||

<!-- markdownlint-enable MD060 -->

* crate without `html_root_url`: [`cargo_metadata::MetadataCommand`]

## Code Blocks

All code block syntaxes in [CommonMark Spec][commonmark-spec] are supported.

In rendered Rust code blocks, `cargo-sync-rdme` matches the hidden-line handling of `rustdoc` for `#`-prefixed lines.

### Fenced code block

**Source:**

````markdown
```
# fn main() {
println!("Hello, world!");
# }
```
````

**Rendered:**

````rust
println!("Hello, world!");
````

### Indented code block

**Source:**

````markdown
    # fn main() {
    println!("Hello, world!");
    # }
````

**Rendered:**

````rust
println!("Hello, world!");
````

## `rustdoc` Markdown Extensions

`cargo-sync-rdme` preserves several Markdown extensions supported by `rustdoc`.

<!-- markdownlint-disable MD060 -->

|Extension|Example|
|---------|-------|
|Tables|This section itself starts with a table.|
|Footnotes|Footnotes work in prose too.[^markdown-extension-footnote]|
|Strikethrough|~~Deprecated wording~~|
|Task lists|See the checklist below.|

<!-- markdownlint-enable MD060 -->

* [x] Completed task list item
* [ ] Incomplete task list item

`rustdoc` also applies smart punctuation, and `cargo-sync-rdme` preserves
those conversions in synced Markdown so README output matches `rustdoc`
more closely.

“quoted text”… really – exactly — like this.

In `rustdoc`, that text is rendered with typographic quotes, an ellipsis,
and en/em dashes.

[^markdown-extension-footnote]: In `rustdoc`, footnotes are collected at the end of the rendered Markdown block.

[intra-doc-link]: https://doc.rust-lang.org/rustdoc/write-documentation/linking-to-items-by-name.html
[Struct]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/struct.Struct.html "struct cargo_sync_rdme_example_lib::Struct"
[`Struct`]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/struct.Struct.html "struct cargo_sync_rdme_example_lib::Struct"
[struct-without-backtick]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/struct.Struct.html "struct cargo_sync_rdme_example_lib::Struct"
[struct-with-backtick]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/struct.Struct.html "struct cargo_sync_rdme_example_lib::Struct"
[`crate::Struct`]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/struct.Struct.html "struct cargo_sync_rdme_example_lib::Struct"
[`self::Struct`]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/struct.Struct.html "struct cargo_sync_rdme_example_lib::Struct"
[`Enum`]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/enum.Enum.html "enum cargo_sync_rdme_example_lib::Enum"
[`Trait`]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/trait.Trait.html "trait cargo_sync_rdme_example_lib::Trait"
[`Union`]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/union.Union.html "union cargo_sync_rdme_example_lib::Union"
[`module`]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/module/index.html "mod cargo_sync_rdme_example_lib::module"
[`CONSTANT`]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/constant.CONSTANT.html "constant cargo_sync_rdme_example_lib::CONSTANT"
[`function`]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/fn.function.html "fn cargo_sync_rdme_example_lib::function"
[`Struct::field`]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/struct.Struct.html#structfield.field "field cargo_sync_rdme_example_lib::Struct::field"
[`Enum::Variant`]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/enum.Enum.html#variant.Variant "variant cargo_sync_rdme_example_lib::Enum::Variant"
[`Trait::method`]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/trait.Trait.html#tymethod.method "method cargo_sync_rdme_example_lib::Trait::method"
[`Clone`]: https://doc.rust-lang.org/1.98.1/core/clone/derive.Clone.html "derive core::clone::Clone"
[`STATIC`]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/static.STATIC.html "static cargo_sync_rdme_example_lib::STATIC"
[`declarative_macro`]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/macro.declarative_macro.html "macro cargo_sync_rdme_example_lib::declarative_macro"
[`TypeAlias`]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/type.TypeAlias.html "type cargo_sync_rdme_example_lib::TypeAlias"
[`i32`]: https://doc.rust-lang.org/1.98.1/std/primitive.i32.html "primitive std::i32"
[`function()`]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/fn.function.html "fn cargo_sync_rdme_example_lib::function"
[`declarative_macro!`]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/macro.declarative_macro.html "macro cargo_sync_rdme_example_lib::declarative_macro"
[`declarative_macro!()`]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/macro.declarative_macro.html "macro cargo_sync_rdme_example_lib::declarative_macro"
[declarative_macro!\[\]]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/macro.declarative_macro.html "macro cargo_sync_rdme_example_lib::declarative_macro"
[`declarative_macro!{}`]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/macro.declarative_macro.html "macro cargo_sync_rdme_example_lib::declarative_macro"
[`crate`]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/index.html "mod cargo_sync_rdme_example_lib"
[`std`]: https://doc.rust-lang.org/1.98.1/std/index.html "mod std"
[`std::collections`]: https://doc.rust-lang.org/1.98.1/std/collections/index.html "mod std::collections"
[`num::bigint`]: https://docs.rs/num/0.4/num/bigint/index.html "mod num::bigint"
[`std::collections::HashMap`]: https://doc.rust-lang.org/1.98.1/std/collections/hash/map/struct.HashMap.html "struct std::collections::hash::map::HashMap"
[num::BigInt@1]: https://docs.rs/num-bigint/0.4/num_bigint/bigint/struct.BigInt.html "struct num_bigint::bigint::BigInt"
[`std::range::Range::start`]: https://doc.rust-lang.org/1.98.1/core/range/struct.Range.html#structfield.start "field core::range::Range::start"
[`num::Complex::re`]: https://docs.rs/num-complex/0.4/num_complex/struct.Complex.html#structfield.re "field num_complex::Complex::re"
[`TupleStruct::0`]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/struct.TupleStruct.html#structfield.0 "field cargo_sync_rdme_example_lib::TupleStruct::0"
[`std::cmp::Reverse::0`]: https://doc.rust-lang.org/1.98.1/core/cmp/struct.Reverse.html#structfield.0 "field core::cmp::Reverse::0"
[`std::mem::MaybeUninit`]: https://doc.rust-lang.org/1.98.1/core/mem/maybe_uninit/union.MaybeUninit.html "union core::mem::maybe_uninit::MaybeUninit"
[`Union::field1`]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/union.Union.html#structfield.field1 "field cargo_sync_rdme_example_lib::Union::field1"
[`Option`]: https://doc.rust-lang.org/1.98.1/core/option/enum.Option.html "enum core::option::Option"
[`num::traits::FloatErrorKind`]: https://docs.rs/num-traits/0.2/num_traits/enum.FloatErrorKind.html "enum num_traits::FloatErrorKind"
[`Option::Some`]: https://doc.rust-lang.org/1.98.1/core/option/enum.Option.html#variant.Some "variant core::option::Option::Some"
[`num::traits::FloatErrorKind::Empty`]: https://docs.rs/num-traits/0.2/num_traits/enum.FloatErrorKind.html#variant.Empty "variant num_traits::FloatErrorKind::Empty"
[`Enum::Struct::field`]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/enum.Enum.html#variant.Struct.field.field "field cargo_sync_rdme_example_lib::Enum::Struct::field"
[`Enum::Tuple::0`]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/enum.Enum.html#variant.Tuple.field.0 "field cargo_sync_rdme_example_lib::Enum::Tuple::0"
[`Option::Some::0`]: https://doc.rust-lang.org/1.98.1/core/option/enum.Option.html#variant.Some.field.0 "field core::option::Option::Some::0"
[`serde::de::Unexpected::Other::0`]: https://docs.rs/serde_core/1.0.229/serde_core/de/enum.Unexpected.html#variant.Other.field.0 "field serde_core::de::Unexpected::Other::0"
[`std::fmt::Result`]: https://doc.rust-lang.org/1.98.1/core/fmt/type.Result.html "type core::fmt::Result"
[`num::BigRational`]: https://docs.rs/num-rational/0.4/num_rational/type.BigRational.html "type num_rational::BigRational"
[`Iterator`]: https://doc.rust-lang.org/1.98.1/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator"
[`num::Num`]: https://docs.rs/num-traits/0.2/num_traits/trait.Num.html "trait num_traits::Num"
[`Iterator::next`]: https://doc.rust-lang.org/1.98.1/core/iter/traits/iterator/trait.Iterator.html#tymethod.next "method core::iter::traits::iterator::Iterator::next"
[`num::Zero::is_zero`]: https://docs.rs/num-traits/0.2/num_traits/identities/trait.Zero.html#tymethod.is_zero "method num_traits::identities::Zero::is_zero"
[`Trait::provided_method`]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/trait.Trait.html#method.provided_method "method cargo_sync_rdme_example_lib::Trait::provided_method"
[`Iterator::size_hint`]: https://doc.rust-lang.org/1.98.1/core/iter/traits/iterator/trait.Iterator.html#tymethod.size_hint "method core::iter::traits::iterator::Iterator::size_hint"
[`num::Zero::set_zero`]: https://docs.rs/num-traits/0.2/num_traits/identities/trait.Zero.html#tymethod.set_zero "method num_traits::identities::Zero::set_zero"
[`Trait::assoc_fn`]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/trait.Trait.html#tymethod.assoc_fn "associated function cargo_sync_rdme_example_lib::Trait::assoc_fn"
[`FromIterator::from_iter`]: https://doc.rust-lang.org/1.98.1/core/iter/traits/collect/trait.FromIterator.html#tymethod.from_iter "method core::iter::traits::collect::FromIterator::from_iter"
[`num::Zero::zero`]: https://docs.rs/num-traits/0.2/num_traits/identities/trait.Zero.html#tymethod.zero "method num_traits::identities::Zero::zero"
[`Trait::provided_assoc_fn`]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/trait.Trait.html#method.provided_assoc_fn "associated function cargo_sync_rdme_example_lib::Trait::provided_assoc_fn"
[`std::iter::Step::forward`]: https://doc.rust-lang.org/1.98.1/core/iter/range/trait.Step.html#tymethod.forward "method core::iter::range::Step::forward"
[`num::FromPrimitive::from_i32`]: https://docs.rs/num-traits/0.2/num_traits/cast/trait.FromPrimitive.html#tymethod.from_i32 "method num_traits::cast::FromPrimitive::from_i32"
[`Trait::CONST`]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/trait.Trait.html#associatedconstant.CONST "associated constant cargo_sync_rdme_example_lib::Trait::CONST"
[`num::traits::ConstZero::ZERO`]: https://docs.rs/num-traits/0.2/num_traits/identities/trait.ConstZero.html#associatedconstant.ZERO "associated constant num_traits::identities::ConstZero::ZERO"
[`Trait::Type`]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/trait.Trait.html#associatedtype.Type "associated type cargo_sync_rdme_example_lib::Trait::Type"
[`Iterator::Item`]: https://doc.rust-lang.org/1.98.1/core/iter/traits/iterator/trait.Iterator.html#associatedtype.Item "associated type core::iter::traits::iterator::Iterator::Item"
[`num::Num::FromStrRadixErr`]: https://docs.rs/num-traits/0.2/num_traits/trait.Num.html#associatedtype.FromStrRadixErr "associated type num_traits::Num::FromStrRadixErr"
[`Struct::method`]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/struct.Struct.html#method.method "method cargo_sync_rdme_example_lib::Struct::method"
[`Vec::clone`]: https://doc.rust-lang.org/1.98.1/alloc/vec/struct.Vec.html#method.clone "method alloc::vec::Vec::clone"
[`num::BigInt::is_zero`]: https://docs.rs/num-bigint/0.4/num_bigint/bigint/struct.BigInt.html#method.is_zero "method num_bigint::bigint::BigInt::is_zero"
[`Struct::provided_method`]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/struct.Struct.html#method.provided_method "method cargo_sync_rdme_example_lib::Struct::provided_method"
[`std::slice::Iter::size_hint`]: https://doc.rust-lang.org/1.98.1/core/slice/iter/struct.Iter.html#method.size_hint "method core::slice::iter::Iter::size_hint"
[`num::BigInt::set_zero`]: https://docs.rs/num-bigint/0.4/num_bigint/bigint/struct.BigInt.html#method.set_zero "method num_bigint::bigint::BigInt::set_zero"
[`Struct::assoc_fn`]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/struct.Struct.html#method.assoc_fn "associated function cargo_sync_rdme_example_lib::Struct::assoc_fn"
[`Vec::from_iter`]: https://doc.rust-lang.org/1.98.1/alloc/vec/struct.Vec.html#method.from_iter "method alloc::vec::Vec::from_iter"
[`num::BigInt::zero`]: https://docs.rs/num-bigint/0.4/num_bigint/bigint/struct.BigInt.html#method.zero "method num_bigint::bigint::BigInt::zero"
[`Struct::CONST`]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/struct.Struct.html#associatedconstant.CONST "associated constant cargo_sync_rdme_example_lib::Struct::CONST"
[`Struct::Type`]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/struct.Struct.html#associatedtype.Type "associated type cargo_sync_rdme_example_lib::Struct::Type"
[`std::slice::Iter::Item`]: https://doc.rust-lang.org/1.98.1/core/slice/iter/struct.Iter.html#associatedtype.Item "associated type core::slice::iter::Iter::Item"
[`num::BigInt::FromStrRadixErr`]: https://docs.rs/num-bigint/0.4/num_bigint/bigint/struct.BigInt.html#associatedtype.FromStrRadixErr "associated type num_bigint::bigint::BigInt::FromStrRadixErr"
[`Struct::inhr_method`]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/struct.Struct.html#method.inhr_method "method cargo_sync_rdme_example_lib::Struct::inhr_method"
[`Vec::len`]: https://doc.rust-lang.org/1.98.1/alloc/vec/struct.Vec.html#method.len "method alloc::vec::Vec::len"
[`num::BigInt::sign`]: https://docs.rs/num-bigint/0.4/num_bigint/bigint/struct.BigInt.html#method.sign "method num_bigint::bigint::BigInt::sign"
[`Struct::inhr_assoc_fn`]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/struct.Struct.html#method.inhr_assoc_fn "associated function cargo_sync_rdme_example_lib::Struct::inhr_assoc_fn"
[`Vec::new`]: https://doc.rust-lang.org/1.98.1/alloc/vec/struct.Vec.html#method.new "method alloc::vec::Vec::new"
[`num::BigInt::new`]: https://docs.rs/num-bigint/0.4/num_bigint/bigint/struct.BigInt.html#method.new "method num_bigint::bigint::BigInt::new"
[`Struct::INHR_CONST`]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/struct.Struct.html#associatedconstant.INHR_CONST "associated constant cargo_sync_rdme_example_lib::Struct::INHR_CONST"
[`std::time::Duration::ZERO`]: https://doc.rust-lang.org/1.98.1/core/time/struct.Duration.html#associatedconstant.ZERO "associated constant core::time::Duration::ZERO"
[num::BigInt::ZERO@1]: https://docs.rs/num-bigint/0.4/num_bigint/bigint/struct.BigInt.html#associatedconstant.ZERO "associated constant num_bigint::bigint::BigInt::ZERO"
[`std::path::MAIN_SEPARATOR`]: https://doc.rust-lang.org/1.98.1/std/path/constant.MAIN_SEPARATOR.html "constant std::path::MAIN_SEPARATOR"
[`std::iter::from_fn`]: https://doc.rust-lang.org/1.98.1/core/iter/sources/from_fn/fn.from_fn.html "fn core::iter::sources::from_fn::from_fn"
[`num::abs`]: https://docs.rs/num-traits/0.2/num_traits/sign/fn.abs.html "fn num_traits::sign::abs"
[`i32::count_ones`]: https://doc.rust-lang.org/1.98.1/std/primitive.i32.html#method.count_ones "method std::i32::count_ones"
[`i32::from_str_radix`]: https://doc.rust-lang.org/1.98.1/std/primitive.i32.html#method.from_str_radix "method std::i32::from_str_radix"
[`i32::MAX`]: https://doc.rust-lang.org/1.98.1/std/primitive.i32.html#associatedconstant.MAX "associated constant std::i32::MAX"
[`println`]: https://doc.rust-lang.org/1.98.1/std/macro.println.html "macro std::println"
[`derive`]: https://doc.rust-lang.org/1.98.1/core/macros/builtin/attr.derive.html "attr core::macros::builtin::derive"
[`async_trait::async_trait`]: https://docs.rs/async-trait/0.1.92/async_trait/attr.async_trait.html "attr async_trait::async_trait"
[`serde::Serialize`]: https://docs.rs/serde_derive/1.0.229/serde_derive/derive.Serialize.html "derive serde_derive::Serialize"
[`ReexportedFromPrivateMod`]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/private/struct.ReexportedFromPrivateMod.html "struct cargo_sync_rdme_example_lib::private::ReexportedFromPrivateMod"
[`foreign_function`]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/fn.foreign_function.html "fn cargo_sync_rdme_example_lib::foreign_function"
[`FOREIGN_STATIC`]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/static.FOREIGN_STATIC.html "static cargo_sync_rdme_example_lib::FOREIGN_STATIC"
[`cargo_metadata::MetadataCommand`]: https://docs.rs/cargo_metadata/0.23.1/cargo_metadata/struct.MetadataCommand.html "struct cargo_metadata::MetadataCommand"
[commonmark-spec]: https://spec.commonmark.org/0.31.2/
<!-- cargo-sync-rdme ]] -->

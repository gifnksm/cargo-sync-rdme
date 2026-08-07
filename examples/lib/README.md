<!-- cargo-sync-rdme title [[ -->
# cargo-sync-rdme-example-lib
<!-- cargo-sync-rdme ]] -->
<!-- cargo-sync-rdme badge -->
<!-- cargo-sync-rdme rustdoc [[ -->
Example library of `cargo-sync-rdme`.

This is document comments embedded in the source code.
It will be extracted and used to generate README.md.

## Intra-doc link support

Intra-doc links are also supported.

### Supported Syntax

[All rustdoc syntax for intra-doc links][intra-doc-link] is supported.

#### Source code

````markdown
* Normal link: [the struct](Struct)
* Normal with backtick link: [the struct](`Struct`)
* Reference link: [the enum][e1]
* Reference link with backtick: [the enum][e2]
* Reference shortcut link: [Union]
* Reference shortcut link with backtick: [`Union`]

* Link with paths: [`crate::Struct`], [`self::Struct`]
* Link with namespace: [`Struct`](struct@Struct), [`declarative_macro`](macro@declarative_macro)
* Link with disambiguators: [`function()`], [`declarative_macro!`]

[e1]: Enum
[e2]: `Enum`
````

#### Rendered

* Normal link: [the struct](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/struct.Struct.html)

* Normal with backtick link: [the struct](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/struct.Struct.html)

* Reference link: [the enum][e1]

* Reference link with backtick: [the enum][e2]

* Reference shortcut link: [Union](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/union.Union.html)

* Reference shortcut link with backtick: [`Union`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/union.Union.html)

* Link with paths: [`crate::Struct`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/struct.Struct.html), [`self::Struct`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/struct.Struct.html)

* Link with namespace: [`Struct`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/struct.Struct.html), [`declarative_macro`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/macro.declarative_macro.html)

* Link with disambiguators: [`function()`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/fn.function.html), [`declarative_macro!`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/macro.declarative_macro.html)

### Link showcase

<!-- markdownlint-disable MD060 -->

|Link Target|[`crate`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/index.html)|[`std`](https://doc.rust-lang.org/nightly/std/index.html)|External Crate|
|-----------|-------|-----|--------------|
|Module|[`module`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/module/index.html)|[`std::collections`](https://doc.rust-lang.org/nightly/std/collections/index.html)|[`num::bigint`](https://docs.rs/num/0.4/num/bigint/index.html)|
|Struct|[`Struct`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/struct.Struct.html)|[`std::collections::HashMap`](https://doc.rust-lang.org/nightly/std/collections/hash/map/struct.HashMap.html)|[`num::BigInt`](https://docs.rs/num-bigint/0.4/num_bigint/bigint/struct.BigInt.html)|
|Struct Field|[`Struct::field`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/struct.Struct.html#structfield.field)|[`std::ops::Range::start`](https://doc.rust-lang.org/nightly/core/ops/range/struct.Range.html#structfield.start)|[`num::Complex::re`](https://docs.rs/num-complex/0.4/num_complex/struct.Complex.html#structfield.re)|
|Tuple Struct Field|[`TupleStruct::0`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/struct.TupleStruct.html#structfield.0)|[`std::cmp::Reverse::0`](https://doc.rust-lang.org/nightly/core/cmp/struct.Reverse.html#structfield.0)||
|Union|[`Union`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/union.Union.html)|[`std::mem::MaybeUninit`](https://doc.rust-lang.org/nightly/core/mem/maybe_uninit/union.MaybeUninit.html)||
|Union Field|[`Union::x`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/union.Union.html#structfield.x)|||
|Enum|[`Enum`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/enum.Enum.html)|[`Option`](https://doc.rust-lang.org/nightly/core/option/enum.Option.html)|[`num::traits::FloatErrorKind`](https://docs.rs/num-traits/0.2/num_traits/enum.FloatErrorKind.html)|
|Enum Variant|[`Enum::Variant`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/enum.Enum.html#variant.Variant)|[`Option::Some`](https://doc.rust-lang.org/nightly/core/option/enum.Option.html#variant.Some)|[`num::traits::FloatErrorKind::Empty`](https://docs.rs/num-traits/0.2/num_traits/enum.FloatErrorKind.html#variant.Empty)|
|Variant Field|[`Enum::Struct::field`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/enum.Enum.html#variant.Struct.field.field)|||
|Tuple Variant Field|[`Enum::Tuple::0`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/enum.Enum.html#variant.Tuple.field.0)|[`Option::Some::0`](https://doc.rust-lang.org/nightly/core/option/enum.Option.html#variant.Some.field.0)|[`serde::de::Unexpected::Other::0`](https://docs.rs/serde_core/1.0.229/serde_core/de/enum.Unexpected.html#variant.Other.field.0)|
|Type Alias|[`TypeAlias`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/type.TypeAlias.html)|[`std::io::Result`](https://doc.rust-lang.org/nightly/core/io/error/type.Result.html)|[`num::BigRational`](https://docs.rs/num-rational/0.4/num_rational/type.BigRational.html)|
|Trait|[`Trait`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/trait.Trait.html)|[`Iterator`](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html)|[`num::Num`](https://docs.rs/num-traits/0.2/num_traits/trait.Num.html)|
|Required Method|[`Trait::method`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/trait.Trait.html#tymethod.method)|[`Iterator::next`](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#tymethod.next)|[`num::Zero::is_zero`](https://docs.rs/num-traits/0.2/num_traits/identities/trait.Zero.html#tymethod.is_zero)|
|Provided Method|[`Trait::provided_method`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/trait.Trait.html#method.provided_method)|[`Iterator::size_hint`](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#tymethod.size_hint)|[`num::Zero::set_zero`](https://docs.rs/num-traits/0.2/num_traits/identities/trait.Zero.html#tymethod.set_zero)|
|Required Associated Function|[`Trait::assoc_fn`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/trait.Trait.html#tymethod.assoc_fn)|[`FromIterator::from_iter`](https://doc.rust-lang.org/nightly/core/iter/traits/collect/trait.FromIterator.html#tymethod.from_iter)|[`num::Zero::zero`](https://docs.rs/num-traits/0.2/num_traits/identities/trait.Zero.html#tymethod.zero)|
|Required Associated Constant|[`Trait::CONST`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/trait.Trait.html#associatedconstant.CONST)||[`num::traits::ConstZero::ZERO`](https://docs.rs/num-traits/0.2/num_traits/identities/trait.ConstZero.html#associatedconstant.ZERO)|
|Required Associated Type|[`Trait::Type`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/trait.Trait.html#associatedtype.Type)|[`Iterator::Item`](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#associatedtype.Item)|[`num::Num::FromStrRadixErr`](https://docs.rs/num-traits/0.2/num_traits/trait.Num.html#associatedtype.FromStrRadixErr)|
|Trait Implementation Method|[`Struct::method`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/struct.Struct.html#method.method)|[`Vec::clone`](https://doc.rust-lang.org/nightly/alloc/vec/struct.Vec.html#method.clone)|[`num::BigInt::is_zero`](https://docs.rs/num-bigint/0.4/num_bigint/bigint/struct.BigInt.html#method.is_zero)|
|Trait Implementation Method (overrides default)|[`Struct::provided_method`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/struct.Struct.html#method.provided_method)|[`std::slice::Iter::size_hint`](https://doc.rust-lang.org/nightly/core/slice/iter/struct.Iter.html#method.size_hint)|[`num::BigInt::set_zero`](https://docs.rs/num-bigint/0.4/num_bigint/bigint/struct.BigInt.html#method.set_zero)|
|Trait Implementation Associated Function|[`Struct::assoc_fn`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/struct.Struct.html#method.assoc_fn)|[`Vec::from_iter`](https://doc.rust-lang.org/nightly/alloc/vec/struct.Vec.html#method.from_iter)|[`num::BigInt::zero`](https://docs.rs/num-bigint/0.4/num_bigint/bigint/struct.BigInt.html#method.zero)|
|Trait Implementation Associated Constant|[`Struct::CONST`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/struct.Struct.html#associatedconstant.CONST)|||
|Trait Implementation Associated Type|[`Struct::Type`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/struct.Struct.html#associatedtype.Type)|[`std::slice::Iter::Item`](https://doc.rust-lang.org/nightly/core/slice/iter/struct.Iter.html#associatedtype.Item)|[`num::BigInt::FromStrRadixErr`](https://docs.rs/num-bigint/0.4/num_bigint/bigint/struct.BigInt.html#associatedtype.FromStrRadixErr)|
|Inherent Method|[`Struct::inhr_method`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/struct.Struct.html#method.inhr_method)|[`Vec::len`](https://doc.rust-lang.org/nightly/alloc/vec/struct.Vec.html#method.len)|[`num::BigInt::sign`](https://docs.rs/num-bigint/0.4/num_bigint/bigint/struct.BigInt.html#method.sign)|
|Inherent Associated Function|[`Struct::inhr_assoc_fn`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/struct.Struct.html#method.inhr_assoc_fn)|[`Vec::new`](https://doc.rust-lang.org/nightly/alloc/vec/struct.Vec.html#method.new)|[`num::BigInt::new`](https://docs.rs/num-bigint/0.4/num_bigint/bigint/struct.BigInt.html#method.new)|
|Inherent Associated Constant|[`Struct::INHR_CONST`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/struct.Struct.html#associatedconstant.INHR_CONST)|[`i32::MAX`](https://doc.rust-lang.org/nightly/std/primitive.i32.html#associatedconstant.MAX)|[`num::BigInt::ZERO`](https://docs.rs/num-bigint/0.4/num_bigint/bigint/struct.BigInt.html#associatedconstant.ZERO)|
|Constant|[`CONSTANT`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/constant.CONSTANT.html)|[`std::path::MAIN_SEPARATOR`](https://doc.rust-lang.org/nightly/std/path/constant.MAIN_SEPARATOR.html)||
|Static|[`STATIC`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/static.STATIC.html)|||
|Function|[`function`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/fn.function.html)|[`std::iter::from_fn`](https://doc.rust-lang.org/nightly/core/iter/sources/from_fn/fn.from_fn.html)|[`num::abs`](https://docs.rs/num-traits/0.2/num_traits/sign/fn.abs.html)|
|Primitive Type||[`i32`](https://doc.rust-lang.org/nightly/std/primitive.i32.html)||
|Primitive Method||[`i32::count_ones`](https://doc.rust-lang.org/nightly/std/primitive.i32.html#method.count_ones)||
|Primitive Associated Function||[`i32::from_str_radix`](https://doc.rust-lang.org/nightly/std/primitive.i32.html#method.from_str_radix)||
|Primitive Associated Constant||[`i32::MAX`](https://doc.rust-lang.org/nightly/std/primitive.i32.html#associatedconstant.MAX)||
|Declarative Macro|[`declarative_macro`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/macro.declarative_macro.html)|[`println`](https://doc.rust-lang.org/nightly/std/macro.println.html)||
|Attribute Macro||[`derive`](https://doc.rust-lang.org/nightly/core/macros/builtin/attr.derive.html)|[`async_trait::async_trait`](https://docs.rs/async-trait/0.1.91/async_trait/attr.async_trait.html)|
|Derive Macro||[`Clone`](https://doc.rust-lang.org/nightly/core/clone/derive.Clone.html)|[`serde::Serialize`](https://docs.rs/serde_derive/1.0.229/serde_derive/derive.Serialize.html)|
|Re-exported from Private Module|[`ReexportedFromPrivateMod`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/private/struct.ReexportedFromPrivateMod.html)|||
|Foreign Function|[`foreign_function`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/fn.foreign_function.html)|||
|Foreign Static|[`FOREIGN_STATIC`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/static.FOREIGN_STATIC.html)|||

<!-- markdownlint-enable MD060 -->

#### Code Block

Fenced code block:

````rust
println!("Hello, world!");
````

Indented code block:

````rust
println!("Hello, world!");

````

[intra-doc-link]: https://doc.rust-lang.org/rustdoc/write-documentation/linking-to-items-by-name.html
[e1]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/enum.Enum.html
[e2]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/enum.Enum.html
<!-- cargo-sync-rdme ]] -->

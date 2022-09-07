<!-- cargo-sync-rdme title [[ -->
# cargo-sync-rdme-example-lib
<!-- cargo-sync-rdme ]] -->
<!-- cargo-sync-rdme badge -->
<!-- cargo-sync-rdme rustdoc [[ -->
Example library of \[`cargo-sync-rdme`\]

This is document comments embedded in the source code.
It will be extracted and used to generate README.md.

# Intra-doc link support

Intra-doc links are also supported.

## Supported Syntax

[All rustdoc syntax for intra-doc links](https://doc.rust-lang.org/rustdoc/write-documentation/linking-to-items-by-name.html) is supported.

### Source code

````markdown
- [Struct]
- [`Struct`]
- [the union](Union)
- [the union](`Union`)
- [the enum][e]

[e]: Enum
````

### Rendered

* [Struct](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/struct.Struct.html)
* [`Struct`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/struct.Struct.html)
* [the union](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/union.Union.html)
* [the union](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/union.Union.html)
* [the enum](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/enum.Enum.html)

## Link showcase

|Item Kind|[`crate`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/index.html)|[`std`](https://doc.rust-lang.org/nightly/std/index.html)|External Crate|
|---------|-------|-----|--------------|
|Module|[`module`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/module/index.html)|[`std::collections`](https://doc.rust-lang.org/nightly/std/collections/index.html)|[`num::bigint`](https://docs.rs/num/0.4/num/bigint/index.html)|
|Struct|[`Struct`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/struct.Struct.html)|[`std::collections::HashMap`](https://doc.rust-lang.org/nightly/std/collections/hash/map/struct.HashMap.html)|[`num::bigint::BigInt`](https://docs.rs/num-bigint/0.4/num_bigint/bigint/struct.BigInt.html)|
|Struct Field [^1]|[`Struct::field`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/struct.Struct.html)|[`std::ops::Range::start`](https://doc.rust-lang.org/nightly/core/ops/range/struct.Range.html)||
|Union|[`Union`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/union.Union.html)|||
|Enum|[`Enum`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/enum.Enum.html)|[`Option`](https://doc.rust-lang.org/nightly/core/option/enum.Option.html)|[`num::traits::FloatErrorKind`](https://docs.rs/num-traits/0.2/num_traits/enum.FloatErrorKind.html)|
|Enum Variant [^2]|[`Enum::Variant`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/enum.Enum.html)|[`Option::Some`](https://doc.rust-lang.org/nightly/core/option/enum.Option.html)|[`num::traits::FloatErrorKind::Empty`](https://docs.rs/num-traits/0.2/num_traits/enum.FloatErrorKind.html)|
|Function|[`function`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/fn.function.html)|[`std::iter::from_fn`](https://doc.rust-lang.org/nightly/core/iter/sources/from_fn/fn.from_fn.html)|[`num::abs`](https://docs.rs/num-traits/0.2/num_traits/sign/fn.abs.html)|
|Typedef|[`Typedef`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/type.Typedef.html)|[`std::io::Result`](https://doc.rust-lang.org/nightly/std/io/error/type.Result.html)|[`num::BigRational`](https://docs.rs/num-rational/0.4/num_rational/type.BigRational.html)|
|Constant|[`CONSTANT`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/constant.CONSTANT.html)|[`std::path::MAIN_SEPARATOR`](https://doc.rust-lang.org/nightly/std/path/constant.MAIN_SEPARATOR.html)||
|Trait|[`Trait`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/trait.Trait.html)|[`std::clone::Clone`](https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html)|[`num::Num`](https://docs.rs/num-traits/0.2/num_traits/trait.Num.html)|
|Method (trait) [^3]|[`Trait::method`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/trait.Trait.html)|[`std::clone::Clone::clone`](https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html)|[`num::Num::from_str_radix`](https://docs.rs/num-traits/0.2/num_traits/trait.Num.html)|
|Method (impl) [^3]|[`Struct::method`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/struct.Struct.html)|[`Vec::clone`](https://doc.rust-lang.org/nightly/alloc/vec/struct.Vec.html)|[`num::bigint::BigInt::from_str_radix`](https://docs.rs/num-bigint/0.4/num_bigint/bigint/struct.BigInt.html)|
|Static|[`STATIC`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/static.STATIC.html)|||
|Macro|[`macro_`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/macro.macro_.html)|[`println`](https://doc.rust-lang.org/nightly/std/macro.println.html)||
|Attribute Macro|||[`async_trait::async_trait`](https://docs.rs/async-trait/0.1.57/async_trait/attr.async_trait.html)|
|Derive Macro|||[`macro@serde::Serialize`](https://docs.rs/serde_derive/1.0.144/serde_derive/derive.Serialize.html)|
|Associated Constant [^4]|[`Trait::CONST`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/trait.Trait.html)|[`i32::MAX`](https://doc.rust-lang.org/nightly/std/primitive.i32.html)||
|Associated Type [^4]|[`Trait::Type`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/trait.Trait.html)|[`Iterator::Item`](https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html)||
|Primitive||[`i32`](https://doc.rust-lang.org/nightly/std/primitive.i32.html)||

[^1]: Intra-doc links to struct fields are not supported in cargo-sync-rdme yet due to [rustdoc bug].

[^2]: Intra-doc links to enum variants are not supported in cargo-sync-rdme yet due to [rustdoc bug].

[^3]: Intra-doc links to methods are not supported in cargo-sync-rdme yet due to [rustdoc bug].

[^4]: Intra-doc links to associated constants or associated types are not supported in cargo-sync-rdme yet due to [rustdoc bug].

[rustdoc bug]: https://github.com/rust-lang/rust/issues/101531
[rustdoc bug]: https://github.com/rust-lang/rust/issues/101531
[rustdoc bug]: https://github.com/rust-lang/rust/issues/101531
[rustdoc bug]: https://github.com/rust-lang/rust/issues/101531
<!-- cargo-sync-rdme ]] -->


<!-- cargo-sync-rdme title [[ -->
# cargo-sync-rdme-example-lib
<!-- cargo-sync-rdme ]] -->
<!-- cargo-sync-rdme badge -->
<!-- cargo-sync-rdme rustdoc [[ -->
Example library of `cargo-sync-rdme`

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
* Link with namespace: [`Struct`](struct@Struct), [`macro_`](macro@macro_)
* Link with disambiguators: [`function()`], [`macro_!`]

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

* Link with namespace: [`Struct`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/struct.Struct.html), [`macro_`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/macro.macro_.html)

* Link with disambiguators: [`function()`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/fn.function.html), [`macro_!`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/macro.macro_.html)

### Link showcase

|Item Kind|[`crate`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/index.html)|[`std`](https://doc.rust-lang.org/nightly/std/index.html)|External Crate|
|---------|-------|-----|--------------|
|Module|[`module`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/module/index.html)|[`std::collections`](https://doc.rust-lang.org/nightly/std/collections/index.html)|[`num::bigint`](https://docs.rs/num/0.4/num/bigint/index.html)|
|Struct|[`Struct`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/struct.Struct.html)|[`std::collections::HashMap`](https://doc.rust-lang.org/nightly/std/collections/hash/map/struct.HashMap.html)|[`num::bigint::BigInt`](https://docs.rs/num-bigint/0.4/num_bigint/bigint/struct.BigInt.html)|
|Struct Field [^1]|[`Struct::field`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/struct.Struct.html#structfield.field)|\[`std::ops::Range::start`\]||
|Union|[`Union`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/union.Union.html)|||
|Enum|[`Enum`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/enum.Enum.html)|[`Option`](https://doc.rust-lang.org/nightly/core/option/enum.Option.html)|[`num::traits::FloatErrorKind`](https://docs.rs/num-traits/0.2/num_traits/enum.FloatErrorKind.html)|
|Enum Variant [^2]|[`Enum::Variant`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/enum.Enum.html#variant.Variant)|\[`Option::Some`\]|\[`num::traits::FloatErrorKind::Empty`\]|
|Function|[`function`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/fn.function.html)|[`std::iter::from_fn`](https://doc.rust-lang.org/nightly/core/iter/sources/from_fn/fn.from_fn.html)|[`num::abs`](https://docs.rs/num-traits/0.2/num_traits/sign/fn.abs.html)|
|Typedef|[`Typedef`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/type.Typedef.html)|[`std::io::Result`](https://doc.rust-lang.org/nightly/std/io/error/type.Result.html)|[`num::BigRational`](https://docs.rs/num-rational/0.4/num_rational/type.BigRational.html)|
|Constant|[`CONSTANT`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/constant.CONSTANT.html)|[`std::path::MAIN_SEPARATOR`](https://doc.rust-lang.org/nightly/std/path/constant.MAIN_SEPARATOR.html)||
|Trait|[`Trait`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/trait.Trait.html)|[`std::clone::Clone`](https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html)|[`num::Num`](https://docs.rs/num-traits/0.2/num_traits/trait.Num.html)|
|Method (trait) [^3]|[`Trait::method`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/Trait/fn.method.html)|\[`std::clone::Clone::clone`\]|\[`num::Num::from_str_radix`\]|
|Method (impl) [^3]|\[`Struct::method`\]|\[`Vec::clone`\]|\[`num::bigint::BigInt::from_str_radix`\]|
|Static|[`STATIC`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/static.STATIC.html)|||
|Macro|[`macro_`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/macro.macro_.html)|[`println`](https://doc.rust-lang.org/nightly/std/macro.println.html)||
|Attribute Macro|||[`async_trait::async_trait`](https://docs.rs/async-trait/0.1.89/async_trait/attr.async_trait.html)|
|Derive Macro|||[`serde::Serialize`](https://docs.rs/serde_derive/1.0.228/serde_derive/derive.Serialize.html)|
|Associated Constant [^4]|[`Trait::CONST`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/trait.Trait.html#associatedconstant.CONST)|\[`i32::MAX`\]||
|Associated Type [^4]|[`Trait::Type`](https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/trait.Trait.html#associatedtype.Type)|\[`Iterator::Item`\]||
|Primitive||[`i32`](https://doc.rust-lang.org/nightly/std/primitive.i32.html)||

[^1]: Intra-doc links to struct fields are not supported in cargo-sync-rdme yet due to [rustdoc bug].

[^2]: Intra-doc links to enum variants are not supported in cargo-sync-rdme yet due to [rustdoc bug].

[^3]: Intra-doc links to methods are not supported in cargo-sync-rdme yet due to [rustdoc bug].

[^4]: Intra-doc links to associated constants or associated types are not supported in cargo-sync-rdme yet due to [rustdoc bug].

#### Code Block

Fenced code block:

````rust
println!("Hello, world!");
````

Indented code blcok:

````rust
println!("Hello, world!");

````

[intra-doc-link]: https://doc.rust-lang.org/rustdoc/write-documentation/linking-to-items-by-name.html
[e1]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/enum.Enum.html
[e2]: https://gifnksm.github.io/cargo-sync-rdme/cargo_sync_rdme_example_lib/enum.Enum.html
[rustdoc bug]: https://github.com/rust-lang/rust/issues/101687
<!-- cargo-sync-rdme ]] -->


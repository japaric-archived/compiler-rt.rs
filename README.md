[![Build Status][status]](https://travis-ci.org/japaric/compiler-rt.rs)

[status]: https://travis-ci.org/japaric/compiler-rt.rs.svg?branch=master

# `compiler-rt.rs`

> Build compiler-rt using build.rs but without relying on its CMake build system.

**WARNING** Barely tested, use this at your own risk!

## Why?

Rust uses LLVM for code generation and LLVM may sometimes "lower" some Rust code to "intrinsics"
which are provided by the `limcompiler-rt.a` library. For "built-in" targets (the ones in
`rustc --print target-list`) there are official binaries of the `libcompiler-rt.a` library, but for
"custom" targets (the ones that need a .json specification file) one has to manually cross compile
the library.

Cross compiling compiler-rt is annoying in the cases where it actually works as it requires `cmake`
and `llvm-config` to be installed. But for custom targets it usually doesn't work because
compiler-rt's CMake build system doesn't work with triples like `thumbv7m-none-eabi` even though
they are valid LLVM targets.

This repository is an experiment about building compiler-rt without using the CMake build system it
ships, with the goal of making it easy to cross compile compiler-rt to custom Rust targets. If
this experiments succeed, I'll use this crate or the ideas developed here in my [Xargo] project as
it's currently lacking a way to cross compile compiler-rt for custom targets.

[Xargo]: https://github.com/japaric/xargo

## Usage

- This crate must appear somewhere in your crate dependency graph. (\*)
- You do **not** need to add `extern crate compiler_rt` anywhere.
- For custom targets, make sure that the `no-compiler-rt` field is set to `false`, which is the
default. If the field is missing from your specification file, that's OK.
- The `linker` field must be set to `$prefix-gcc` **or** the variables `CC_${TARGET//-/_}` and
`AR_${TARGET//-/_}`  must be set to `$prefix-gcc` and `$prefix-ar` respectively. If both are set,
the env variables take precedence.

(\*) It's unclear to me what happens if *two different* versions of this crate appear in your
dependency graph. Cargo will probably raise an error at link time.

Example for the custom target `thumbv7m-none-eabi`

```
$ tail -n2 Cargo.toml
[dependencies.compiler-rt]
git = "https://github.com/japaric/compiler-rt.rs"

# `no-compiler-rt` is not set
$ grep no-compiler-rt cortex-m3.json

$ grep linker cortex-m3.json
    "linker": "arm-none-eabi-gcc"
    
$ cargo build --target cortex-m3
(..)
   Compiling compiler-rt v0.1.0
(..)
   
$ find -name '*.a'
./target/cortex-m3/debug/build/compiler-rt-d33efb9ff92c364e/out/libcompiler-rt.a
```

## Caveats

[caveats]: #caveats

- Doesn't work with all the built-in targets. No real blocker for this; it just has to be
implemented and tested.
- Requires `git` to be in your `$PATH`. This requirement will be lifted in the future.
- Requires a nightly `rustc` because this crate is `no_core`, but it may make sense to make this
crate `no_std` to make it usable with other channels.

[0]: https://github.com/rust-lang/rust/pull/32988

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

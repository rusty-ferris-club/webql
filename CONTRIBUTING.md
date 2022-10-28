# Contributing code to webql

Everyone is welcome to contribute code to `webql`, provided that they are willing to license their contributions under the same license as the project itself.
We follow a simple 'inbound=outbound' model for contributions: the act of submitting an 'inbound' contribution means that the contributor agrees to license the code under the same terms as the project's overall 'outbound' license - in this case, Apache Software License v2 (see [LICENSE](./LICENSE)).


## How to contribute

The preferred and easiest way to contribute changes to the project is to fork it on GitHub, and then create a pull request to ask us to pull your changes into our repo.

Things that should go into your PR description:

 - References to any bugs fixed by the change
 - Notes for the reviewer that might help them to understand why the change is necessary or how they might better review it

Your PR must also:

 - be based on the `main` branch
 - adhere to the [code style](#code-style)
 - pass the [test suites](#tests)
 - check [documentation](#documentation)


## Tests

In `webql` we have few test suite flows that need to pass before merging to master.
- [unitest](#unitest)
- [clippy](#clippy)
- [rustfmt](#rustfmt)

### unitest

run the following command:
```bash
cargo xtask test
```

Include feature flag:
```bash
cargo xtask test -- --features github
```

To capture the snapshots test we using [insta](https://github.com/mitsuhiko/insta) rust project. you can see the snapshot changes / new snapshot by running the command:
```bash
cargo insta test --review
```

### clippy
```bash
cargo xtask clippy
```

### rustfmt
```bash
cargo xtask fmt
```

## Code style

We use the standard Rust code style, and enforce it with `rustfmt`/`cargo fmt`.
A few code style options are set in the [`.rustfmt.toml`](./.rustfmt.toml) file, and some of them are not stable yet and require a nightly version of rustfmt.


## documentation

Generate and open [webql](https://github.com/rusty-ferris-club/webql) to make sure that your documentation os current

```bash
cargo xtask docs-preview
```
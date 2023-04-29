## Rust basics

* what is `"xx".to_own()`? Looks converting `&str` to `String`?
* closures (anonymous functions)

    ```rust
    let func = |x: i32, y: i32| { (x+y)>3 };
    ```

## crates

### `tracing`

* log format (here):

    ```
    [LEVEL crate::src-file] content
    ```

    * configure through environment variable `RA_LOG`

* `debug!`, `info!` etc: https://docs.rs/tracing/latest/tracing/#for-log-users
* new "operators" like `?` and `%`: https://docs.rs/tracing/latest/tracing/#recording-fields
    * btw, native Rust operators: https://doc.rust-lang.org/book/appendix-02-operators.html

### `anyhow`

don't care about callee error types

```toml
[dependencies]
anyhow = <version>
```

```rust
fn main() -> anyhow::Result<()> {
    let x: i32 = "      123   ".trim().parse()?;
    println!("{x}");

    let x: i32 = "not a number".trim().parse()?;
    println!("{x}");

    Ok(())
}
```
## rustc

```shell
rustc +nightly -Z unstable-options --print target-spec-json
```

interpretation of the `data-layout` field: https://stackoverflow.com/questions/67888518/how-do-i-understand-the-data-layout-strings-of-rust-compiler-targets

## project-specific

### misc

```rust
toolchain::cargo() // get cargo path
toolchain::rustc() // get rustc path
```

### meta

* websites
    * 主仓库 CI 构建的是 API 文档
    * manual 在另一个仓库，可能是在手动同步...? https://github.com/rust-analyzer/rust-analyzer.github.io
* 版本与 changelog
    * `Cargo.toml` 里全是 `0.0.0`
    * 类似 `0.3.1489` 的版本号可能来自 [`dist.rs`](./xtask/src/dist.rs)
    * see also [`changelog.rs`](./xtask/src/release/changelog.rs), [`version.rs`](./crates/rust-analyzer/src/version.rs)

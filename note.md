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

## project-specific

```rust
toolchain::cargo() // get cargo path
toolchain::rustc() // get rustc path
```

Adhoc Errors
============

[![Latest Version](https://img.shields.io/crates/v/adhocerr.svg)](https://crates.io/crates/adhocerr)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/adhocerr)

A library for the construction of efficient single use static/dynamic error types per callsite.


```toml
[dependencies]
adhocerr = "0.1"
```

<br>

## Examples

Creating an root cause error:

```rust
use adhocerr::err;

fn get_git_root(start: &Path) -> Result<PathBuf, impl Error + 'static> {
    start
        .ancestors()
        .find(|a| a.join(".git").is_dir())
        .map(Path::to_owned)
        .ok_or(err!("Unable to find .git/ in parent directories"))
}
```

Wrapping another Error:

```rust
use adhocerr::wrap;

fn record_success() -> Result<(), impl Error + 'static> {
    std::fs::write(".success", "true").map_err(wrap!("Failed to save results of script"))
}
```

<br>

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>


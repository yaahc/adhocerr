//! A library for the construction of efficient static/dynamic single use error types per callsite.
//!
//!
//! ```toml
//! [dependencies]
//! adhocerr = "0.1"
//! ```
//!
//! ## Examples
//!
//! Creating a root cause error:
//!
//! ```rust
//! use adhocerr::err;
//! # use std::{
//! #     error::Error,
//! #     path::{Path, PathBuf},
//! # };
//!
//! fn get_git_root(start: &Path) -> Result<PathBuf, impl Error + 'static> {
//!     start
//!         .ancestors()
//!         .find(|a| a.join(".git").is_dir())
//!         .map(Path::to_owned)
//!         .ok_or(err!("Unable to find .git/ in parent directories"))
//! }
//! ```
//!
//! Wrapping another Error:
//!
//! ```rust
//! use adhocerr::wrap;
//! # use std::error::Error;
//!
//! fn record_success() -> Result<(), impl Error + 'static> {
//!     std::fs::write(".success", "true").map_err(wrap!("Failed to save results of script"))
//! }
//! ```
//!
//! ## Details
//!
//! This crate provides two primary macros. `err!` and `wrap!`. The former, `err!`, is used to
//! create ad-hoc error types without a root cause from strings. `wrap!` on the other hand is used
//! to create new errors with a source member.
//!
//! Both of these macros have two versions, and they generate completely different code, depending
//! on whether or not string interopoation (`format!`-like code) is used in the error message. When
//! the error message is a fixed string, the macro declares a new struct in line that has the
//! string itself inserted into its `Display` implementation. This way no memory is used or
//! allocations made to hold the error when they are not needed.
//!
//! For `err!` this means that your error type is a Zero Sized Type (ZST), for
//! `wrap!` this means that your Wrapper error is the same size as the original
//! error you're wrapping.
//!
//! When runtime interpolation is used and a `String` allocation is necessary it
//! uses pre defined Error types to wrap the String to avoid declaring new types
//! unnecessarily, but hides them behind an impl Trait boundary.
//!
//! ### Expanded
//!
//! The Expanded version of the example above would look like this:
//!
//! ```rust
//! # use std::{
//! #     error::Error,
//! #     path::{Path, PathBuf},
//! # };
//! fn get_git_root(start: &Path) -> Result<PathBuf, impl Error + 'static> {
//!     start
//!         .ancestors()
//!         .find(|a| a.join(".git").is_dir())
//!         .map(Path::to_owned)
//!         .ok_or({
//!             #[derive(Debug)]
//!             struct AdhocError;
//!
//!             impl std::error::Error for AdhocError {
//!                 fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
//!                     None
//!                 }
//!             }
//!
//!             impl core::fmt::Display for AdhocError {
//!                 fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
//!                     f.write_str("Unable to find .git/ in parent directories")
//!                 }
//!             }
//!
//!             AdhocError
//!         })
//! }
//! ```
use core::fmt;
pub use err as format_err;

/// Thinly wrap an error by defining a hidden error type and returning a closure to construct it
///
/// ## Examples
///
/// Wrap an error without changing its size or allocating:
///
/// ```rust
/// use adhocerr::wrap;
/// # use std::error::Error;
///
/// fn record_success() -> Result<(), impl Error + 'static> {
///     std::fs::write(".success", "true").map_err(wrap!("Failed to save results of script"))
/// }
/// ```
///
/// Which expands to:
///
///
/// ```rust
/// # use std::error::Error;
/// fn record_success() -> Result<(), impl Error + 'static> {
///     std::fs::write(".success", "true").map_err({
///         #[derive(Debug)]
///         struct WrappedError<E> {
///             source: E,
///         }
///
///         impl<E> std::error::Error for WrappedError<E>
///         where
///             E: std::error::Error + 'static,
///         {
///             fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
///                 Some(&self.source)
///             }
///         }
///
///         impl<E> core::fmt::Display for WrappedError<E>
///         where
///             E: std::error::Error + 'static,
///         {
///             fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
///                 f.write_str("Failed to save results of script")
///             }
///         }
///
///         |source| WrappedError { source }
///     })
/// }
/// ```
///
/// Wrapping an error with an runtime generated String:
///
/// ```rust
/// use adhocerr::wrap;
/// # use std::{error::Error, path::Path};
///
/// fn record_success(file: &Path) -> Result<(), impl Error + 'static> {
///     std::fs::write(file, "true").map_err(wrap!(
///         "Failed to save results of script to file: {}",
///         file.display()
///     ))
/// }
/// ```
///
/// Which expands to:
///
/// ```rust
/// # use std::{error::Error, path::Path};
/// fn record_success(file: &Path) -> Result<(), impl Error + 'static> {
///     std::fs::write(file, "true").map_err(|source| {
///         adhocerr::private::format_wrap_err(
///             source,
///             format_args!(
///                 "Failed to save results of script to file: {}",
///                 file.display()
///             ),
///         )
///     })
/// }
/// ```
#[macro_export]
macro_rules! wrap {
    ($msg:literal) => {{
        #[derive(Debug)]
        struct WrappedError<E> {
            source: E,
        }

        impl<E> std::error::Error for WrappedError<E>
        where
            E: std::error::Error + 'static,
        {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                Some(&self.source)
            }
        }

        impl<E> core::fmt::Display for WrappedError<E>
        where
            E: std::error::Error + 'static,
        {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str($msg)
            }
        }

        |source| WrappedError { source }
    }};
    ($fmt:literal, $($arg:tt)*) => {
        |source| $crate::private::format_wrap_err(source, format_args!($fmt, $($arg)*))
    };
}

/// Create an ad-hoc error type with zero size if none is needed
///
/// ## Examples
///
/// Creating a static ad-hoc error type:
///
/// ```rust
/// use adhocerr::err;
/// # use std::{
/// #     error::Error,
/// #     path::{Path, PathBuf},
/// # };
///
/// fn get_git_root(start: &Path) -> Result<PathBuf, impl Error + 'static> {
///     start
///         .ancestors()
///         .find(|a| a.join(".git").is_dir())
///         .map(Path::to_owned)
///         .ok_or(err!("Unable to find .git/ in parent directories"))
/// }
/// ```
///
/// Which expands to:
///
/// ```rust
/// # use std::{
/// #     error::Error,
/// #     path::{Path, PathBuf},
/// # };
/// fn get_git_root(start: &Path) -> Result<PathBuf, impl Error + 'static> {
///     start
///         .ancestors()
///         .find(|a| a.join(".git").is_dir())
///         .map(Path::to_owned)
///         .ok_or({
///             #[derive(Debug)]
///             struct AdhocError;
///
///             impl std::error::Error for AdhocError {
///                 fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
///                     None
///                 }
///             }
///
///             impl core::fmt::Display for AdhocError {
///                 fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
///                     f.write_str("Unable to find .git/ in parent directories")
///                 }
///             }
///
///             AdhocError
///         })
/// }
/// ```
///
/// Creating a dynamic ad-hoc error type:
///
/// ```rust
/// use adhocerr::err;
/// # use std::{
/// #     error::Error,
/// #     path::{Path, PathBuf},
/// # };
///
/// fn get_git_root(start: &Path) -> Result<PathBuf, impl Error + 'static> {
///     start
///         .ancestors()
///         .find(|a| a.join(".git").is_dir())
///         .map(Path::to_owned)
///         .ok_or(err!(
///             "Unable to find .git/ in parent directories for {}",
///             start.display()
///         ))
/// }
/// ```
///
/// Which expands to:
///
/// ```rust
/// # use std::{
/// #     error::Error,
/// #     path::{Path, PathBuf},
/// # };
/// fn get_git_root(start: &Path) -> Result<PathBuf, impl Error + 'static> {
///     start
///         .ancestors()
///         .find(|a| a.join(".git").is_dir())
///         .map(Path::to_owned)
///         .ok_or(adhocerr::private::format_err(format_args!(
///             "Unable to find .git/ in parent directories for {}",
///             start.display()
///         )))
/// }
/// ```
#[macro_export]
macro_rules! err {
    ($msg:literal) => {{
        #[derive(Debug)]
        struct AdhocError;

        impl std::error::Error for AdhocError {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                None
            }
        }

        impl core::fmt::Display for AdhocError {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str($msg)
            }
        }

        AdhocError
    }};
    ($fmt:literal, $($arg:tt)*) => {
        $crate::private::format_err(format_args!($fmt, $($arg)*))
    };
}

#[derive(Debug)]
struct FormatError(String);

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for FormatError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

#[derive(Debug)]
struct FormatWrappedError<E> {
    msg: String,
    source: E,
}

impl<E> fmt::Display for FormatWrappedError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.msg.fmt(f)
    }
}

impl<E> std::error::Error for FormatWrappedError<E>
where
    E: std::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

/// Return early with an error if a condition is not satisfied.
///
/// This macro is equivalent to `if !$cond { return Err(From::from($err)); }`.
///
/// Analogously to `assert!`, `ensure!` takes a condition and exits the function
/// if the condition fails. Unlike `assert!`, `ensure!` returns an `Error`
/// rather than panicking.
///
/// # Example
///
/// ```
/// use adhocerr::ensure;
///
/// fn main() -> Result<(), impl std::error::Error + 'static> {
/// #     let user = 0;
/// #
///     ensure!(user == 0, "only user 0 is allowed");
///     Ok(())
/// }
/// ```
///
/// ```
/// use adhocerr::ensure;
///
/// # const MAX_DEPTH: usize = 1;
/// #
/// fn main() -> Result<(), anyhow::Error> {
/// #     let depth = 0;
/// #
///     ensure!(depth <= MAX_DEPTH, "Recursion limit exceeded"?);
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $msg:literal) => {
        if !$cond {
            return $crate::private::Err($crate::err!($msg));
        }
    };
    ($cond:expr, $msg:literal?) => {
        if !$cond {
            return $crate::private::Err($crate::err!($msg).into());
        }
    };
    ($cond:expr, $fmt:literal, $($arg:tt)*) => {
        if !$cond {
            return $crate::private::Err($crate::err!($fmt, $($arg)*));
        }
    };
    ($cond:expr, $fmt:literal, $($arg:tt)*?) => {
        if !$cond {
            return $crate::private::Err($crate::err!($fmt, $($arg)*).into());
        }
    };
}

/// Return an ad-hoc error immediately
///
/// This macro is equivalent to `return Err(From::from($err))`.
///
/// # Example
///
/// ```
/// use adhocerr::bail;
///
/// # fn has_permission(user: usize, resource: usize) -> bool {
/// #     true
/// # }
/// #
/// fn main() -> Result<(), impl std::error::Error + 'static> {
/// #     let user = 0;
/// #     let resource = 0;
/// #
///     if !has_permission(user, resource) {
///         bail!("permission denied for accessing {}", resource);
///     }
///
///     Ok(())
/// }
/// ```
///
/// ```
/// use adhocerr::bail;
///
/// # const MAX_DEPTH: usize = 1;
/// #
/// fn main() -> Result<(), anyhow::Error> {
/// #     let depth = 0;
/// #
///     if depth > MAX_DEPTH {
///         bail!("Recursion limit exceeded"?);
///     }
///
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! bail {
    ($msg:literal) => {
        return $crate::private::Err($crate::err!($msg));
    };
    ($msg:literal?) => {
        return $crate::private::Err($crate::err!($msg).into());
    };
    ($fmt:literal, $($arg:tt)*) => {
        return $crate::private::Err($crate::err!($fmt, $($arg)*));
    };
    ($fmt:literal, $($arg:tt)*?) => {
        return $crate::private::Err($crate::err!($fmt, $($arg)*).into());
    };
}

#[doc(hidden)]
pub mod private {
    pub use core::result::Result::Err;

    pub fn format_err(
        args: core::fmt::Arguments<'_>,
    ) -> impl std::error::Error + Send + Sync + 'static {
        crate::FormatError(args.to_string())
    }

    pub fn format_wrap_err(
        source: impl std::error::Error + Send + Sync + 'static,
        args: core::fmt::Arguments<'_>,
    ) -> impl std::error::Error + Send + Sync + 'static {
        crate::FormatWrappedError {
            msg: args.to_string(),
            source,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn try_code_anyhow() -> Result<(), anyhow::Error> {
        let code = 1;

        ensure!(code == 0, "Command exited with a non zero status code"?);

        Ok(())
    }

    fn try_code_impl() -> Result<(), impl std::error::Error + 'static> {
        let code = 1;

        ensure!(code == 0, "Command exited with a non zero status code");

        Ok(())
    }

    #[test]
    #[should_panic]
    fn ensure_coerce() {
        try_code_anyhow().unwrap();
    }

    #[test]
    #[should_panic]
    fn ensure_impl() {
        try_code_impl().unwrap();
    }
}

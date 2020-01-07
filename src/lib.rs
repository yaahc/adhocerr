pub use err as format_err;
use std::fmt;

/// Thinly wrap an error by defining a hidden error type and returning a closure to construct it
#[macro_export]
macro_rules! wrap_err {
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

        impl<E> std::fmt::Display for WrappedError<E>
        where
            E: std::error::Error + 'static,
        {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str($msg)
            }
        }

        |source| WrappedError { source }
    }};
    ($fmt:literal, $($arg:tt)*) => {
        |source| $crate::private::format_wrap_err(source, format_args!($fmt, $($arg)*))
    };
}

/// Create an adhoc error type with zero size if none is needed
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

        impl std::fmt::Display for AdhocError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

#[macro_export]
macro_rules! ensure {
    ($cond:expr, $msg:literal $(,)?) => {
        if !$cond {
            return $crate::err!($msg);
        }
    };
    ($cond:expr, $fmt:literal, $($arg:tt)*) => {
        if !$cond {
            return $crate::err!($fmt, $($arg)*);
        }
    };
}

#[macro_export]
macro_rules! bail {
    ($msg:literal $(,)?) => {
        return $crate::private::Err($crate::err!($msg));
    };
    ($fmt:literal, $($arg:tt)*) => {
        return $crate::private::Err($crate::err!($fmt, $($arg)*));
    };
}

#[doc(hidden)]
pub mod private {
    pub use core::result::Result::Err;

    pub fn format_err(
        args: std::fmt::Arguments<'_>,
    ) -> impl std::error::Error + Send + Sync + 'static {
        crate::FormatError(args.to_string())
    }

    pub fn format_wrap_err(
        source: impl std::error::Error + Send + Sync + 'static,
        args: std::fmt::Arguments<'_>,
    ) -> impl std::error::Error + Send + Sync + 'static {
        crate::FormatWrappedError {
            msg: args.to_string(),
            source,
        }
    }
}

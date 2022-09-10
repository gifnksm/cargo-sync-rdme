#![allow(unused_macros, unused_macro_rules)]

// workaround for https://github.com/zkat/miette/issues/201
macro_rules! bail {
    ($msg:literal $(,)?) => {
        {
            return miette::private::Err(miette!($msg));
        }
    };
    ($err:expr $(,)?) => {
        {
            return miette::private::Err(miette!($err));
        }
    };
    ($fmt:expr, $($arg:tt)*) => {
        {
            return miette::private::Err(miette!($fmt, $($arg)*));
        }
    };
}

// workaround for https://github.com/zkat/miette/issues/201
macro_rules! ensure {
    ($cond:expr, $msg:literal $(,)?) => {
        if !$cond {
            return miette::private::Err($crate::miette!($msg));
        }
    };
    ($cond:expr, $err:expr $(,)?) => {
        if !$cond {
            return miette::private::Err($crate::miette!($err));
        }
    };
    ($cond:expr, $fmt:expr, $($arg:tt)*) => {
        if !$cond {
            return miette::private::Err($crate::miette!($fmt, $($arg)*));
        }
    };
}

// workaround for https://github.com/zkat/miette/issues/201
macro_rules! miette {
    ($msg:literal $(,)?) => {
        miette::private::new_adhoc(format!($msg))
    };
    ($err:expr $(,)?) => ({
        use miette::private::kind::*;
        let error = $err;
        (&error).miette_kind().new(error)
    });
    ($fmt:expr, $($arg:tt)*) => {
        miette::private::new_adhoc(format!($fmt, $($arg)*))
    };
}

/// try for iterator implementation
macro_rules! itry {
    ($e:expr) => {
        match $e {
            Ok(v) => v,
            Err(e) => {
                return Some(Err(e));
            }
        }
    };
}

macro_rules! opt_try {
    ($e:expr) => {
        match $e {
            Some(v) => v,
            None => {
                return Ok(None);
            }
        }
    };
}

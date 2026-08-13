/// try for iterator implementation.
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

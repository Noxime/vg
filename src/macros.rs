macro_rules! log {
    () => (log_!("\n"));
    ($fmt:expr) => (log_!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (log_!(concat!($fmt, "\n"), $($arg)*));
}

macro_rules! log_ {
    ($($arg:tt)*) => ($crate::arch::stdout(format_args!($($arg)*)));
}

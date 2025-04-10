pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}

#[macro_export]
macro_rules! log {
    (error, $($arg:tt)*) => {
        $crate::print_color!("31", concat!("[error] ", $($arg)*, "\n"));
    };
    (warn, $($arg:tt)*) => {
        $crate::print_color!("33", concat!("[warn] ", $($arg)*, "\n"));
    };
    (info, $($arg:tt)*) => {
        $crate::print_color!("32", concat!("[info] ", $($arg)*, "\n"));
    };
    (debug, $($arg:tt)*) => {
        $crate::print_color!("34", concat!("[debug] ", $($arg)*, "\n"));
    };
}
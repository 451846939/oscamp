//! Standard library macros

/// Prints to the standard output.
///
/// Equivalent to the [`println!`] macro except that a newline is not printed at
/// the end of the message.
///
/// [`println!`]: crate::println
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::io::__print_impl(format_args!($($arg)*));
    }
}

/// Prints to the standard output, with a newline.
#[macro_export]
macro_rules! println {
    () => { $crate::print!("\n") };
    ($($arg:tt)*) => {
        $crate::io::__print_impl(format_args!("{}\n", format_args!($($arg)*)));
    }
}

#[macro_export]
macro_rules! print_color {
    ($color:expr, $($arg:tt)*) => {{
        use axstd::io::Write;

        let mut out = $crate::io::stdout().lock();
        let _ = write!(out, "\x1B[{}m", $color); // 开启颜色
        let _ = write!(out, $($arg)*);           // 打印内容
        let _ = write!(out, "\x1B[0m");          // 重置颜色
    }};
}
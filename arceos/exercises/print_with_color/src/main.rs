#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[cfg(feature = "axstd")]
use axstd::log;


#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    log!(info,"[WithColor]: Hello, Arceos!");
}

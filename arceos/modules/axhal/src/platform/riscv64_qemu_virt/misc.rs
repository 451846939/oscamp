use axlog::ax_println;

/// Shutdown the whole system, including all CPUs.
pub fn terminate() -> ! {
    ax_println!("Shutting down...");
    sbi_rt::system_reset(sbi_rt::Shutdown, sbi_rt::NoReason);
    warn!("It should shutdown!");
    loop {
        crate::arch::halt();
    }
}

#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

extern crate alloc;
#[cfg(feature = "axstd")]
extern crate axstd as std;

#[macro_use]
extern crate axlog;

mod loader;
mod syscall;
mod task;

use alloc::sync::Arc;
use core::result;
use axhal::arch::UspaceContext;
use axhal::cpu::current_task_ptr;
use axhal::mem::VirtAddr;
use axhal::paging::MappingFlags;
use axhal::trap::{register_trap_handler, PAGE_FAULT};
use axmm::AddrSpace;
use axstd::io;
use axsync::Mutex;
use axtask::TaskExtRef;
use loader::load_user_app;

const USER_STACK_SIZE: usize = 0x10000;
const KERNEL_STACK_SIZE: usize = 0x40000; // 256 KiB
const APP_ENTRY: usize = 0x1000;

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    // A new address space for user app.
    let mut uspace = axmm::new_user_aspace().unwrap();

    // Load user app binary file into address space.
    if let Err(e) = load_user_app("/sbin/origin", &mut uspace) {
        panic!("Cannot load app! {:?}", e);
    }

    // Init user stack.
    let ustack_top = init_user_stack(&mut uspace, false).unwrap();
    ax_println!("New user address space: {:#x?}", uspace);

    // Let's kick off the user process.
    let user_task = task::spawn_user_task(
        Arc::new(Mutex::new(uspace)),
        UspaceContext::new(APP_ENTRY.into(), ustack_top),
    );

    // Wait for user process to exit ...
    let exit_code = user_task.join();
    ax_println!("monolithic kernel exit [{:?}] normally!", exit_code);
}

fn init_user_stack(uspace: &mut AddrSpace, populating: bool) -> io::Result<VirtAddr> {
    let ustack_top = uspace.end();
    let ustack_vaddr = ustack_top - crate::USER_STACK_SIZE;
    ax_println!(
        "Mapping user stack: {:#x?} -> {:#x?}",
        ustack_vaddr,
        ustack_top
    );
    uspace
        .map_alloc(
            ustack_vaddr,
            crate::USER_STACK_SIZE,
            MappingFlags::READ | MappingFlags::WRITE | MappingFlags::USER,
            populating,
        )
        .unwrap();
    Ok(ustack_top)
}

#[register_trap_handler(PAGE_FAULT)]
fn page_fault_handler(fault_addr: VirtAddr, access_flags: MappingFlags, is_user: bool) -> bool {
    if is_user {
        let result=axtask::current().task_ext().aspace.lock().handle_page_fault(fault_addr, access_flags);
        if result {
            ax_println!("Page fault handled successfully: {:#x?}", fault_addr);
        } else {
            ax_println!("Failed to handle page fault: {:#x?}", fault_addr);
            // Handle the error (e.g., terminate the task)
            axtask::exit(-1);
        }
        true
    } else {
        false
    }
}

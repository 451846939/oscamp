//! Memory mapping backends.
#![allow(dead_code)]

use axhal::paging::{MappingFlags, PageTable};
use memory_addr::{MemoryAddr, VirtAddr, PAGE_SIZE_4K};
use memory_set::MappingBackend;
use axalloc::global_allocator;
use axhal::mem::virt_to_phys;

mod alloc;
mod linear;

/// A unified enum type for different memory mapping backends.
///
/// Currently, two backends are implemented:
///
/// - **Linear**: used for linear mappings. The target physical frames are
///   contiguous and their addresses should be known when creating the mapping.
/// - **Allocation**: used in general, or for lazy mappings. The target physical
///   frames are obtained from the global allocator.
#[derive(Clone)]
pub enum Backend {
    /// Linear mapping backend.
    ///
    /// The offset between the virtual address and the physical address is
    /// constant, which is specified by `pa_va_offset`. For example, the virtual
    /// address `vaddr` is mapped to the physical address `vaddr - pa_va_offset`.
    Linear {
        /// `vaddr - paddr`.
        pa_va_offset: usize,
    },
    /// Allocation mapping backend.
    ///
    /// If `populate` is `true`, all physical frames are allocated when the
    /// mapping is created, and no page faults are triggered during the memory
    /// access. Otherwise, the physical frames are allocated on demand (by
    /// handling page faults).
    Alloc {
        /// Whether to populate the physical frames when creating the mapping.
        populate: bool,
    },
    /// File-backed mapping backend (lazy load).
    FileBacked {
        reader: ::alloc::sync::Arc<dyn crate::MmapReadFn>,
        file_offset: usize,
        area_start: VirtAddr,
    },

}

impl MappingBackend for Backend {
    type Addr = VirtAddr;
    type Flags = MappingFlags;
    type PageTable = PageTable;
    fn map(&self, start: VirtAddr, size: usize, flags: MappingFlags, pt: &mut PageTable) -> bool {
        match *self {
            Self::Linear { pa_va_offset } => self.map_linear(start, size, flags, pt, pa_va_offset),
            Self::Alloc { populate } => self.map_alloc(start, size, flags, pt, populate),
            Self::FileBacked { .. } =>
                true, // Don't map on creation, rely on lazy fault handler
        }
    }

    fn unmap(&self, start: VirtAddr, size: usize, pt: &mut PageTable) -> bool {
        match *self {
            Self::Linear { pa_va_offset } => self.unmap_linear(start, size, pt, pa_va_offset),
            Self::Alloc { populate } => self.unmap_alloc(start, size, pt, populate),
            Self::FileBacked { .. } =>
                pt.unmap_region(start, size, true).is_ok(),
        }
    }

    fn protect(
        &self,
        start: Self::Addr,
        size: usize,
        new_flags: Self::Flags,
        page_table: &mut Self::PageTable,
    ) -> bool {
        page_table
            .protect_region(start, size, new_flags, true)
            .map(|tlb| tlb.ignore())
            .is_ok()
    }
}

impl Backend {

    pub fn new_file_backed(
        reader: ::alloc::sync::Arc<dyn crate::MmapReadFn>,
        file_offset: usize,
        area_start: VirtAddr,
    ) -> Self {
        Self::FileBacked {
            reader,
            file_offset,
            area_start,
        }
    }
    pub(crate) fn handle_page_fault(
        &self,
        vaddr: VirtAddr,
        orig_flags: MappingFlags,
        page_table: &mut PageTable,
    ) -> bool {
        match self {
            Self::Linear { .. } => false, // Linear mappings should not trigger page faults.
            Self::Alloc { populate } => {
                self.handle_page_fault_alloc(vaddr, orig_flags, page_table, *populate)
            }
            Self::FileBacked {
                reader,
                file_offset,
                area_start,
            } => {
                let va = vaddr.align_down(PAGE_SIZE_4K);
                let offset = file_offset + (va.as_usize() - area_start.as_usize());

                // 分配一页虚拟内存
                let vaddr = match global_allocator().alloc_pages(1, PAGE_SIZE_4K) {
                    Ok(vaddr) => vaddr,
                    Err(_) => return false,
                };

                let paddr = virt_to_phys(VirtAddr::from(vaddr));
                let buf = unsafe {
                    core::slice::from_raw_parts_mut(axhal::mem::phys_to_virt(paddr).as_mut_ptr(), PAGE_SIZE_4K)
                };
                if !(reader)(offset, buf) {
                    return false;
                }

                page_table
                    .map_region(
                        va,
                        |_| paddr,
                        PAGE_SIZE_4K,
                        orig_flags,
                        false,
                        false,
                    )
                    .map(|tlb| tlb.ignore())
                    .is_ok()
            }
        }
    }
}

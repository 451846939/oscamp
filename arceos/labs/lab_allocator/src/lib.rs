#![feature(allocator_api)]
//! Allocator algorithm in lab.

#![no_std]
#![allow(unused_variables)]

use allocator::{BaseAllocator, ByteAllocator, AllocResult};
use core::ptr::NonNull;
use core::alloc::{Layout};
use allocator::AllocError;
const MIN_BUCKET: usize = 5; // 2^5 = 32 bytes
const MAX_BUCKET: usize = 12; // 2^12 = 4096 bytes
const BUCKET_COUNT: usize = MAX_BUCKET - MIN_BUCKET + 1;
const META_SIZE: usize = core::mem::size_of::<u8>();

#[repr(C)]
struct FreeBlock {
    next: Option<NonNull<FreeBlock>>,
}

pub struct LabByteAllocator {
    heap_start: usize,
    heap_end: usize,
    current: usize,
    buckets: [Option<NonNull<FreeBlock>>; BUCKET_COUNT],
}

impl LabByteAllocator {
    pub const fn new() -> Self {
        const NONE: Option<NonNull<FreeBlock>> = None;
        Self {
            heap_start: 0,
            heap_end: 0,
            current: 0,
            buckets: [NONE; BUCKET_COUNT],
        }
    }

    fn align_up(x: usize, align: usize) -> usize {
        (x + align - 1) & !(align - 1)
    }

    fn bucket_index(size: usize) -> Option<usize> {
        let total = size + META_SIZE;
        let n = total.next_power_of_two();
        let log2 = n.trailing_zeros() as usize;
        if log2 < MIN_BUCKET || log2 > MAX_BUCKET {
            None
        } else {
            Some(log2 - MIN_BUCKET)
        }
    }

    fn bucket_size(index: usize) -> usize {
        1 << (index + MIN_BUCKET)
    }
}

impl BaseAllocator for LabByteAllocator {
    fn init(&mut self, start: usize, size: usize) {
        self.heap_start = start;
        self.current = start;
        self.heap_end = start + size;
    }

    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult {
        if start == self.heap_end {
            self.heap_end += size;
            Ok(())
        } else {
            Err(AllocError::InvalidParam)
        }
    }
}

impl ByteAllocator for LabByteAllocator {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        let size = layout.size().max(layout.align());

        if let Some(index) = Self::bucket_index(size) {
            let alloc_size = Self::bucket_size(index);

            // Try freelist first
            let bucket = &mut self.buckets[index];
            if let Some(mut block_ptr) = *bucket {
                unsafe {
                    let block = block_ptr.as_mut();
                    *bucket = block.next;
                }
                let meta_ptr = block_ptr.as_ptr() as *mut u8;
                let user_ptr = unsafe { meta_ptr.add(META_SIZE) };
                unsafe { *meta_ptr = index as u8 };
                return Ok(NonNull::new(user_ptr).unwrap());
            }

            // Bump allocate
            let alloc_start = Self::align_up(self.current, layout.align());
            let alloc_end = alloc_start.checked_add(alloc_size).ok_or(AllocError::InvalidParam)?;
            if alloc_end > self.heap_end {
                return Err(AllocError::NoMemory);
            }

            self.current = alloc_end;

            let meta_ptr = alloc_start as *mut u8;
            let user_ptr = unsafe { meta_ptr.add(META_SIZE) };
            unsafe { *meta_ptr = index as u8 };
            Ok(NonNull::new(user_ptr).unwrap())
        } else {
            // Large allocation, no metadata
            let alloc_start = Self::align_up(self.current, layout.align());
            let alloc_end = alloc_start.checked_add(layout.size()).ok_or(AllocError::InvalidParam)?;
            if alloc_end > self.heap_end {
                return Err(AllocError::NoMemory);
            }
            self.current = alloc_end;
            Ok(NonNull::new(alloc_start as *mut u8).unwrap())
        }
    }

    fn dealloc(&mut self, ptr: NonNull<u8>, _layout: Layout) {
        let user_ptr = ptr.as_ptr();
        let meta_ptr = unsafe { user_ptr.sub(META_SIZE) };
        let index = unsafe { *meta_ptr } as usize;

        if index >= BUCKET_COUNT {
            return; // large allocation, do not recycle
        }

        let block_ptr = meta_ptr as *mut FreeBlock;
        unsafe {
            (*block_ptr).next = self.buckets[index];
            self.buckets[index] = Some(NonNull::new_unchecked(block_ptr));
        }
    }

    fn total_bytes(&self) -> usize {
        self.heap_end - self.heap_start
    }

    fn used_bytes(&self) -> usize {
        self.current - self.heap_start
    }

    fn available_bytes(&self) -> usize {
        self.heap_end - self.current
    }
}

unsafe impl Send for LabByteAllocator {}
unsafe impl Sync for LabByteAllocator {}
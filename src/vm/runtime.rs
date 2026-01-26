use std::{
    alloc::{Layout, alloc, dealloc},
    ptr::NonNull,
};

pub struct Heap {
    ptr: NonNull<u8>,
    layout: Layout,
    cap: u64,
    cursor: u64,
}

impl Heap {
    pub fn new_with_capacity(cap: u64) -> Self {
        let align = 8;
        let layout = Layout::from_size_align(cap as usize, align)
            .expect("Invalid allocation layout when initializing heap");
        let ptr = unsafe {
            let raw_buf = alloc(layout);
            NonNull::new(raw_buf).expect("Failed to allocate memory for heap")
        };

        let remainder = ptr.addr().get() % align;
        let padding = if remainder == 0 { 0 } else { align - remainder };
        let ptr = unsafe { ptr.add(padding) };

        Self {
            ptr,
            layout,
            cap,
            cursor: padding as u64,
        }
    }

    pub fn alloc(&mut self, size: u64) -> Option<NonNull<u8>> {
        let current = unsafe { self.ptr.add(self.cursor as usize) };

        let align = self.layout.align() as u64;
        let remainder = size % align;
        let padding = if remainder == 0 { 0 } else { align - remainder };
        self.cursor += padding + size;

        if self.cursor > self.cap {
            None
        } else {
            Some(current)
        }
    }
}

impl Drop for Heap {
    fn drop(&mut self) {
        unsafe { dealloc(self.ptr.as_ptr(), self.layout) }
    }
}

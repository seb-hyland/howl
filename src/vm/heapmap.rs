use crate::vm::{
    runtime::{Heap, HeapAllocator},
    value::Value,
};
use hashbrown::raw::RawTable;
use std::ptr::NonNull;

#[repr(C, align(16))]
pub struct HeapMapHeader {
    capacity: u64,
    count: u64,
    ptr: NonNull<u8>,
}

pub struct HeapMap {
    inner_map: HashMap<Value, Value>,
    allocator: HeapAllocator<HeapMapHeader>,
}

impl HeapMap {
    pub fn new(heap: &mut Heap, capacity: u64) -> Self {
        let allocator = HeapAllocator::new(heap);
        let map = HashTable::with_capacity_in(capacity as usize, allocator);

        let map_ptr = map.raw;
    }
}

use crate::{
    IdentArena,
    vm::{
        bytecode::OpCode,
        heapmap::HeapMap,
        value::{TypeId, Value},
    },
};
use std::{
    alloc::{AllocError, Allocator, Layout},
    marker::PhantomData,
    mem::MaybeUninit,
    ptr::NonNull,
};

pub struct Runtime {
    pub stack: Vec<Value>,
    pub code: Vec<OpCode>,
    pub pc: u64,
    pub heap: Heap,
    pub globals: Globals,
}

pub struct Globals {
    pub idents: IdentArena,
    /// id -> val
    pub vars: HeapMap,
    /// id -> <id -> Method>
    pub types: HeapMap,
}

type ExternHandler = fn(rt: &mut Runtime, arg_count: u64) -> Option<Value>;

impl Default for Runtime {
    fn default() -> Self {
        let mut heap = Heap::default();
        let globals = Globals {
            idents: IdentArena::default(),
            vars: HeapMap::new(&mut heap, 4_096),
            types: HeapMap::new(&mut heap, 64),
        };
        let mut rt = Self {
            heap,
            globals,
            pc: 0,
            stack: Vec::with_capacity(30),
            code: Vec::new(),
        };
        crate::std::define_std_types(&mut rt);
        rt
    }
}

impl Runtime {
    #[inline(always)]
    pub fn push_stack(&mut self, v: Value) {
        self.stack.push(v);
    }

    #[inline(always)]
    pub fn pop_stack(&mut self) -> Value {
        self.stack.pop().expect("Stack should not be empty")
    }

    pub fn peek(&self) -> &Value {
        self.stack.last().expect("Stack should not be empty")
    }

    pub fn peek_at(&self, n: u64) -> Value {
        let n = n as usize;
        let len = self.stack.len();
        if len <= n {
            panic!(
                "Stack underflow during peek_at: requested {}, but stack size is {}",
                n, len
            );
        }
        self.stack[len - 1 - n]
    }

    #[inline(always)]
    pub fn pop_stack_n<const N: usize>(&mut self) -> [Value; N] {
        let mut array = [MaybeUninit::uninit(); N];
        for i in (0..N).rev() {
            array[i].write(self.pop_stack());
        }
        unsafe { MaybeUninit::array_assume_init(array) }
    }

    pub fn define_type(&mut self, id: TypeId) {
        let map = HeapMap::new(&mut self.heap, 16);
        self.globals.types.insert(
            Value::from_uint(id as u64),
            Value::from_ptr(map.ptr.as_ptr() as u64),
        );
    }

    pub fn register_handler(
        &mut self,
        name: &'static str,
        handler: ExternHandler,
        type_id: TypeId,
    ) {
        let handler_map_id = self.globals.idents.add(name);
        let handler_ptr = self
            .globals
            .types
            .get(&Value::from_uint(type_id as u64))
            .unwrap()
            .as_ptr();
        let mut handler_map = unsafe {
            HeapMap::from_ptr(
                NonNull::new(handler_ptr as *mut Value).unwrap(),
                &mut self.heap,
            )
        };
        handler_map.insert(
            Value::from_uint(handler_map_id),
            #[allow(clippy::fn_to_numeric_cast)]
            Value::from_uint(handler as u64),
        );
    }

    pub fn push_op(&mut self, op: OpCode) {
        self.code.push(op);
    }
}

pub struct Heap {
    ptr: NonNull<u8>,
    cap: u64,
    cursor: u64,
}

pub struct HeapAllocator<T> {
    inner: NonNull<Heap>,
    _phantom: PhantomData<T>,
}

impl<T> HeapAllocator<T> {
    pub fn new(heap: &mut Heap) -> Self {
        Self {
            inner: NonNull::from_mut(heap),
            _phantom: PhantomData {},
        }
    }
}

#[repr(C)]
pub struct HeapMetadata {
    type_id: TypeId,
    alloc_size: u64,
}

pub struct Allocation {
    header_ptr: NonNull<u8>,
    data_ptr: NonNull<u8>,
}

impl Heap {
    const ALIGN: usize = 16;

    pub fn new_with_capacity(cap: u64) -> Self {
        let layout = Layout::from_size_align(cap as usize, Self::ALIGN)
            .expect("Invalid allocation layout when initializing heap");
        let ptr = unsafe {
            let raw_buf = alloc(layout);
            NonNull::new(raw_buf).expect("Failed to allocate memory for heap")
        };
        Self {
            ptr,
            cap,
            cursor: 0,
        }
    }

    pub fn alloc<DataHeader>(&mut self, layout: Layout, type_id: TypeId) -> Option<Allocation> {
        let metadata_size = size_of::<HeapMetadata>() as u64;
        let header_size = size_of::<DataHeader>() as u64;
        let reserved = metadata_size + header_size;

        let earliest_start = self.cursor + reserved;

        let align = layout.align() as u64;
        let data_offset = (earliest_start + align - 1) & !(align - 1);

        let total_alloc_size = data_offset + layout.size() as u64;
        // We are OOM :<3 (this won't happen due to pre-emptive GC eventually)
        if total_alloc_size > self.cap {
            return None;
        }

        let data_ptr = unsafe { self.ptr.add(data_offset as usize) };
        let header_ptr = unsafe { data_ptr.sub(header_size as usize) };
        unsafe {
            let header = HeapMetadata {
                alloc_size: layout.size() as u64,
                type_id,
            };
            let metadata_ptr = header_ptr.sub(metadata_size as usize);
            metadata_ptr.cast::<HeapMetadata>().write(header);
        }

        self.cursor = total_alloc_size;
        Some(Allocation {
            header_ptr,
            data_ptr,
        })
    }

    pub fn get_alloc_header<Header>(&self, ptr: NonNull<u8>) -> NonNull<Header> {
        unsafe { ptr.sub(size_of::<Header>()).cast() }
    }

    pub fn get_alloc_metadata<Header>(&self, ptr: NonNull<u8>) -> NonNull<HeapMetadata> {
        unsafe {
            ptr.sub(size_of::<Header>() + size_of::<HeapMetadata>())
                .cast()
        }
    }
}

unsafe impl<Header> Allocator for HeapAllocator<Header> {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        unsafe { self.inner.as_ptr().as_mut_unchecked() }
            .alloc::<Header>(layout, TypeId::NONE)
            .map(|allocation| NonNull::slice_from_raw_parts(allocation.data_ptr, layout.size()))
            .ok_or(AllocError)
    }

    unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: Layout) {}
}

impl Default for Heap {
    fn default() -> Self {
        Self::new_with_capacity(32_000_000)
    }
}

impl Drop for Heap {
    fn drop(&mut self) {
        unsafe {
            dealloc(
                self.ptr.as_ptr(),
                Layout::from_size_align(self.cap as usize, Self::ALIGN).unwrap(),
            )
        }
    }
}

use crate::{
    IdentArena,
    vm::{
        bytecode::OpCode,
        value::{TypeId, Value},
    },
};
use std::{
    alloc::{Layout, alloc, dealloc},
    hash::{DefaultHasher, Hash, Hasher},
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

#[derive(Debug)]
pub struct HeapMap {
    pub heap: NonNull<Heap>,
    pub capacity: u64,
    pub count: u64,
    pub ptr: NonNull<Value>,
}

impl HeapMap {
    const NUM_FIELDS: u64 = 2;

    pub fn new(heap: &mut Heap, capacity: u64) -> Self {
        let map = Self {
            heap: NonNull::from_mut(heap),
            capacity,
            count: 0,
            ptr: heap
                .alloc(
                    (capacity * 2 + Self::NUM_FIELDS) * size_of::<Value>() as u64,
                    TypeId::HeapMap,
                )
                .unwrap()
                .cast::<Value>(),
        };

        unsafe {
            map.ptr.write(Value::from_uint(capacity));
            map.ptr.add(1).write(Value::from_uint(0));
        };

        for i in 0..map.capacity {
            unsafe {
                map.ptr
                    .add(i as usize * 2 + Self::NUM_FIELDS as usize)
                    .write(Value::nil())
            };
        }

        map
    }

    pub unsafe fn from_ptr(ptr: NonNull<Value>, heap: &mut Heap) -> Self {
        Self {
            heap: NonNull::from_mut(heap),
            capacity: unsafe { *ptr.as_ptr() }.as_uint(),
            count: unsafe { *ptr.add(1).as_ptr() }.as_uint(),
            ptr,
        }
    }

    fn calculate_index(&self, key: &Value) -> u64 {
        let mut h = DefaultHasher::new();
        key.hash(&mut h);
        h.finish() & (self.capacity - 1)
    }

    pub fn insert(&mut self, k: Value, v: Value) {
        if self.count as f64 / self.capacity as f64 > 0.7 {
            self.realloc();
        }

        let mut idx = self.calculate_index(&k);
        loop {
            let key_ptr = unsafe {
                self.ptr
                    .add(idx as usize * 2 + Self::NUM_FIELDS as usize)
                    .as_ptr()
            };
            if unsafe { *key_ptr } == Value::nil() {
                unsafe {
                    *key_ptr = k;
                    *key_ptr.add(1) = v;
                    self.count += 1;
                    *self.ptr.add(1).as_ptr() = Value::from_uint(self.count);
                }
                break;
            } else if unsafe { *key_ptr } == k {
                unsafe { *key_ptr.add(1) = v };
                break;
            } else {
                idx = (idx + 1) & (self.capacity - 1);
                continue;
            }
        }
    }

    fn realloc(&mut self) {
        todo!();
        let mut new_map = HeapMap::new(unsafe { self.heap.as_mut() }, self.capacity * 2);
        for i in 0..self.capacity {
            let key = unsafe {
                *self
                    .ptr
                    .add(i as usize * 2 + Self::NUM_FIELDS as usize)
                    .as_ptr()
            };
            if key != Value::nil() {
                let value = unsafe { *self.ptr.add(i as usize * 2 + 1).as_ptr() };
                new_map.insert(key, value);
            }
        }
        *self = new_map;
    }

    pub fn get(&self, k: &Value) -> Option<Value> {
        let mut idx = self.calculate_index(k);
        let start_idx = idx;

        loop {
            let key_ptr = unsafe {
                self.ptr
                    .add(idx as usize * 2 + Self::NUM_FIELDS as usize)
                    .as_ptr()
            };
            if unsafe { *key_ptr } == *k {
                let value = unsafe { *key_ptr.add(1) };
                return Some(value);
            } else if unsafe { *key_ptr } == Value::nil() {
                return None;
            }

            idx = (idx + 1) & (self.capacity - 1);
            if idx == start_idx {
                return None;
            }
        }
    }
}

pub struct Heap {
    ptr: NonNull<u8>,
    layout: Layout,
    cap: u64,
    cursor: u64,
}

impl Heap {
    const ALIGN: u64 = 16;

    pub fn new_with_capacity(cap: u64) -> Self {
        let align = Self::ALIGN as usize;

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

    pub fn alloc(&mut self, size: u64, type_id: TypeId) -> Option<NonNull<u8>> {
        let current = unsafe { self.ptr.add(self.cursor as usize) };

        let align = Self::ALIGN;
        let remainder = self.cursor % align;
        let padding = if remainder == 0 { 0 } else { align - remainder };
        let current = unsafe { current.add(padding as usize) };

        let header_size = 16;
        unsafe { current.as_ptr().cast::<TypeId>().write(type_id) };

        self.cursor += padding + size + header_size;
        let current = unsafe { current.add(header_size as usize) };

        if self.cursor > self.cap {
            None
        } else {
            Some(current)
        }
    }
}

impl Default for Heap {
    fn default() -> Self {
        Self::new_with_capacity(32_000_000)
    }
}

impl Drop for Heap {
    fn drop(&mut self) {
        unsafe { dealloc(self.ptr.as_ptr(), self.layout) }
    }
}

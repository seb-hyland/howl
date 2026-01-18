use std::{cell::UnsafeCell, ops::Range};

pub mod lexer;
pub mod parser;

#[derive(PartialEq, Debug, Clone)]
pub struct Span(Range<usize>);

pub struct StateIterator<'src, T> {
    buf: &'src [T],
    current: UnsafeCell<usize>,
}

impl<'src, T> StateIterator<'src, T> {
    fn new(buf: &'src [T]) -> Self {
        Self {
            buf,
            current: UnsafeCell::new(0),
        }
    }

    fn current(&self) -> usize {
        unsafe { *self.current.get() }
    }

    fn peek(&self) -> Option<&T> {
        let current = unsafe { *self.current.get() };
        self.buf.get(current)
    }

    fn advance(&self) -> Option<&T> {
        let current = self.current.get();
        let cur_item = self.buf.get(unsafe { *current });
        if cur_item.is_some() {
            unsafe {
                *current += 1;
            }
        }
        cur_item
    }

    fn peek_from_current(&self) -> impl IntoIterator<Item = (usize, &T)> {
        self.buf[unsafe { *self.current.get() }..]
            .iter()
            .enumerate()
    }

    fn slice_advance(&self, end: usize) -> &[T] {
        if end == 0 {
            todo!()
        }
        unsafe {
            *self.current.get() += end - 1;
        }
        &self.buf[unsafe { *self.current.get() }..end]
    }
}

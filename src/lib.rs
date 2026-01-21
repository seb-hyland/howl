use std::{cell::UnsafeCell, ops::Range};

pub mod lexer;
pub mod parser;
pub mod vm;

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
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

    #[inline(always)]
    fn current(&self) -> usize {
        unsafe { *self.current.get() }
    }

    #[inline(always)]
    fn incr_current(&self) {
        unsafe { *self.current.get() += 1 }
    }

    fn peek(&self) -> Option<&T> {
        self.buf.get(self.current())
    }

    fn advance(&self) -> Option<&T> {
        let cur_item = self.buf.get(self.current());
        if cur_item.is_some() {
            self.incr_current();
        }
        cur_item
    }

    fn peek_from_current(&self) -> impl IntoIterator<Item = (usize, &T)> {
        self.buf[self.current()..].iter().enumerate()
    }

    fn slice_advance_to_end(&self) -> &[T] {
        let current = self.current();
        let end_idx = self.buf.len();
        unsafe { *self.current.get() = end_idx };
        &self.buf[current..end_idx]
    }

    fn slice_advance(&self, end: usize) -> &[T] {
        if end == 0 {
            todo!()
        }
        let current = self.current();

        let slice = &self.buf[current..current + end];
        unsafe { *self.current.get() += end - 1 };
        slice
    }
}

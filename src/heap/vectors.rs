use core::mem::size_of;

use crate::mem::alloc::kalloc;

#[repr(C, packed)]
pub struct Vec<T> {
    pointer: *mut T,
    length: usize,
    capacity: usize,
}

impl<T> Vec<T> {
    pub fn new() -> Self {
        if size_of::<T>() != 0 {
            Vec {
                pointer: kalloc(80 * size_of::<T>()) as *mut T,
                length: 0,
                capacity: 80 * size_of::<T>(),
            }
        } else {
            Vec {
                pointer: kalloc(0) as *mut T,
                length: usize::MAX,
                capacity: 0,
            }
        }
    }

    pub const fn empty_null() -> Self {
        Vec {
            pointer: 0 as *mut T,
            length: 0,
            capacity: 0,
        }
    }

    pub fn push(&self, elem: T) {
        unsafe {
            self.pointer.offset(todo!());
        }
    }
}

#![feature(untagged_unions)]

use std::mem;
use std::ptr;
use std::marker::PhantomData;

fn uninitialized<T>() -> MaybeUninitialized<T> {
    MaybeUninitialized { _uninitialized: () }
}

#[allow(unions_with_drop_fields)]
union MaybeUninitialized<T> {
    data: T,
    _uninitialized: (),
}

pub struct MaybeBox<T> {
    data: usize,
    _ph: PhantomData<T>,
}

/*
#[inline]
unsafe fn transmogrify_inline<'a, T>(ptr: &'a usize) -> &'a T {
    mem::transmute(ptr)
}
*/

#[inline]
unsafe fn transmogrify_inline_mut<'a, T>(ptr: &'a mut usize) -> &'a mut T {
    mem::transmute(ptr)
}

/*
#[inline]
unsafe fn transmogrify_boxed<'a, T>(ptr: &'a usize) -> &'a Box<T> {
    mem::transmute(ptr)
}
*/

#[inline]
unsafe fn transmogrify_boxed_mut<'a, T>(ptr: &'a mut usize) -> &'a mut Box<T> {
    mem::transmute(ptr)
}

unsafe fn new_inline<'a, T>(t: T, ptr: &'a mut usize) {
    let ptr = transmogrify_inline_mut(ptr);
    ptr::write(ptr, t);
}

unsafe fn new_boxed<'a, T>(t: T, ptr: &'a mut usize) {
    let ptr = transmogrify_boxed_mut(ptr);
    ptr::write(ptr, Box::new(t));
}

unsafe fn drop_inline<'a, T>(ptr: &'a mut usize) {
    let ptr = transmogrify_inline_mut(ptr);
    let _: T = ptr::read(ptr);
}

unsafe fn drop_boxed<'a, T>(ptr: &'a mut usize) {
    let ptr = transmogrify_boxed_mut(ptr);
    let _: Box<T> = ptr::read(ptr);
}

impl<T> MaybeBox<T> {
    #[inline]
    pub fn new(t: T) -> MaybeBox<T> {
        let mut new: MaybeUninitialized<MaybeBox<T>> = uninitialized();
        unsafe {
            {
                let ptr = &mut new.data.data;
                if mem::size_of::<T>() <= mem::size_of::<usize>() {
                    new_inline::<T>(t, ptr)
                } else {
                    new_boxed::<T>(t, ptr)
                };
            }
            new.data
        }
    }
}

impl<T> Drop for MaybeBox<T> {
    fn drop(&mut self) {
        let ptr = &mut self.data;
        if mem::size_of::<T>() <= mem::size_of::<usize>() {
            unsafe { drop_inline::<T>(ptr) }
        } else {
            unsafe { drop_boxed::<T>(ptr) }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let t = 123u8;
        let mb = MaybeBox::new(t);
        drop(mb);

        let t = 123usize;
        let mb = MaybeBox::new(t);
        drop(mb);

        let t = String::from("hello");
        let mb = MaybeBox::new(t);
        drop(mb);

        let t = Box::new(123u32);
        let mb = MaybeBox::new(t);
        drop(mb);
    }
}


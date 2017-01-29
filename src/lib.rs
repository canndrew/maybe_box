use std::mem;
use std::ptr;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

pub struct MaybeBox<T> {
    data: usize,
    _ph: PhantomData<T>,
}

#[inline]
unsafe fn transmogrify_inline<'a, T>(ptr: &'a usize) -> &'a T {
    mem::transmute(ptr)
}

#[inline]
unsafe fn transmogrify_inline_mut<'a, T>(ptr: &'a mut usize) -> &'a mut T {
    mem::transmute(ptr)
}

#[inline]
unsafe fn transmogrify_boxed<'a, T>(ptr: &'a usize) -> &'a Box<T> {
    mem::transmute(ptr)
}

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

unsafe fn get_inline<'a, T>(ptr: &'a mut usize) -> T {
    let ptr = transmogrify_inline_mut(ptr);
    let t: T = ptr::read(ptr);
    t
}

unsafe fn get_boxed<'a, T>(ptr: &'a mut usize) -> T {
    let ptr = transmogrify_boxed_mut(ptr);
    let b: Box<T> = ptr::read(ptr);
    *b
}

impl<T> MaybeBox<T> {
    #[inline]
    pub fn new(t: T) -> MaybeBox<T> {
        let mut new: MaybeBox<T> = unsafe { mem::uninitialized() };
        unsafe {
            {
                let ptr = &mut new.data;
                if mem::size_of::<T>() <= mem::size_of::<usize>() {
                    new_inline::<T>(t, ptr)
                } else {
                    new_boxed::<T>(t, ptr)
                };
            }
            new
        }
    }

    pub fn into_inner(mut self) -> T {
        let ret = self.get_inner();
        mem::forget(self);
        ret
    }

    fn get_inner(&mut self) -> T {
        let ptr = &mut self.data;
        if mem::size_of::<T>() <= mem::size_of::<usize>() {
            unsafe { get_inline::<T>(ptr) }
        } else {
            unsafe { get_boxed::<T>(ptr) }
        }
    }
}

impl<T> Drop for MaybeBox<T> {
    fn drop(&mut self) {
        let _: T = self.get_inner();
    }
}

impl<T> From<T> for MaybeBox<T> {
    fn from(t: T) -> MaybeBox<T> {
        MaybeBox::new(t)
    }
}

impl<T> Deref for MaybeBox<T> {
    type Target = T;

    fn deref(&self) -> &T {
        let ptr = &self.data;
        if mem::size_of::<T>() <= mem::size_of::<usize>() {
            unsafe { transmogrify_inline::<T>(ptr) }
        } else {
            &*unsafe { transmogrify_boxed::<T>(ptr) }
        }
    }
}

impl<T> DerefMut for MaybeBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        let ptr = &mut self.data;
        if mem::size_of::<T>() <= mem::size_of::<usize>() {
            unsafe { transmogrify_inline_mut::<T>(ptr) }
        } else {
            &mut *unsafe { transmogrify_boxed_mut::<T>(ptr) }
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
        let mb = MaybeBox::from(t);
        let t = mb.into_inner();
        assert_eq!(t, 123u8);

        let t = 123usize;
        let mb = MaybeBox::new(t);
        drop(mb);
        let mb = MaybeBox::from(t);
        let t = mb.into_inner();
        assert_eq!(t, 123usize);

        let t = String::from("hello");
        let mb = MaybeBox::new(t);
        drop(mb);

        let t = String::from("hello");
        let mb = MaybeBox::from(t);
        let t = mb.into_inner();
        assert_eq!(&t, "hello");

        let t = Box::new(123u32);
        let mb = MaybeBox::new(t);
        drop(mb);

        let t = Box::new(123u32);
        let mb = MaybeBox::from(t);
        let t = mb.into_inner();
        assert_eq!(*t, 123u32);
    }
}


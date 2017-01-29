//! Store arbitrary data in the size of a `usize`, only boxing it if necessary.

use std::mem;
use std::ptr;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::fmt;
use std::hash;

/// Hold a value of type `T` in the space for a `usize`, only boxing it if necessary.
/// This can be a useful optimization when dealing with C APIs that allow you to pass around some
/// arbitrary `void *`-sized piece of data.
///
/// This type is guranteed to be the same size as a `usize`.
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

unsafe fn get_boxed<'a, T>(ptr: &'a mut usize) -> Box<T> {
    let ptr = transmogrify_boxed_mut(ptr);
    let b: Box<T> = ptr::read(ptr);
    b
}

/// An unpacked `MaybeBox<T>`. Produced by `MaybeBox::unpack`.
#[derive(Debug)]
pub enum Unpacked<T> {
    /// A `T` stored inline.
    Inline(T),
    /// A `T` stored in a `Box`.
    Boxed(Box<T>),
}

impl<T> MaybeBox<T> {
    /// Wrap a `T` into a `MaybeBox<T>`. This will allocate if
    /// `size_of::<T>() > size_of::<usize>()`.
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

    /// Consume the `MaybeBox<T>` and return the inner `T`.
    pub fn into_inner(mut self) -> T {
        let ret = self.get_inner();
        mem::forget(self);
        ret
    }

    /// Consume the `MaybeBox<T>` and return the inner `T`, possibly boxed (if
    /// it was already).
    ///
    /// This may be more efficient than calling `into_inner` and then boxing
    /// the returned value.
    pub fn unpack(mut self) -> Unpacked<T> {
        let ret = {
            let ptr = &mut self.data;
            if mem::size_of::<T>() <= mem::size_of::<usize>() {
                Unpacked::Inline(unsafe { get_inline::<T>(ptr) })
            } else {
                Unpacked::Boxed(unsafe { get_boxed::<T>(ptr) })
            }
        };
        mem::forget(self);
        ret
    }

    fn get_inner(&mut self) -> T {
        let ptr = &mut self.data;
        if mem::size_of::<T>() <= mem::size_of::<usize>() {
            unsafe { get_inline::<T>(ptr) }
        } else {
            *unsafe { get_boxed::<T>(ptr) }
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

impl<T: fmt::Debug> fmt::Debug for MaybeBox<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let inner: &T = &**self;
        f.debug_tuple("MaybeBox").field(inner).finish()
    }
}

impl<U, T: PartialEq<U>> PartialEq<MaybeBox<U>> for MaybeBox<T> {
    fn eq(&self, other: &MaybeBox<U>) -> bool {
        let l: &T = &**self;
        let r: &U = &**other;
        *l == *r
    }

    fn ne(&self, other: &MaybeBox<U>) -> bool {
        let l: &T = &**self;
        let r: &U = &**other;
        *l != *r
    }
}

impl<T: Eq> Eq for MaybeBox<T> {}

impl<T: hash::Hash> hash::Hash for MaybeBox<T> {
    fn hash<H>(&self, state: &mut H)
        where H: hash::Hasher
    {
        let inner: &T = &**self;
        T::hash(inner, state)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        assert_eq!(std::mem::size_of::<usize>(),
                   std::mem::size_of::<MaybeBox<u32>>());

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

        let t = true;
        let mb = MaybeBox::new(t);
        assert_eq!(format!("{:?}", mb), "MaybeBox(true)");
        match mb.unpack() {
            Unpacked::Inline(true) => (),
            x => panic!("Unexpected!: {:?}", x),
        };

        let t = String::from("hello");
        let mb = MaybeBox::new(t);
        match mb.unpack() {
            Unpacked::Boxed(b) => {
                assert_eq!(&*b, "hello");
            },
            x => panic!("Unexpected!: {:?}", x),
        };
    }
}


//! Dynamically-atomic reference-counting pointers.
//!
//! This is a proof of concept of a Rust `Rc<T>` type that can *dynamically* choose
//! whether to use atomic access to update its reference count. A related `Arc<T>`
//! can be created which offers thread-safe (`Send + Sync`) access to the same
//! data. If there's never an `Arc`, the `Rc` never pays the price for atomics.

use std::cell::{Cell, UnsafeCell};
use std::isize;
use std::marker::PhantomData;
use std::ops::Deref;
use std::process::abort;
use std::ptr::NonNull;
use std::sync::atomic::{self, AtomicUsize, Ordering};

/// A soft limit on the amount of references that may be made to an `Arc`.
///
/// Going above this limit will abort your program (although not
/// necessarily) at _exactly_ `MAX_REFCOUNT + 1` references.
const MAX_REFCOUNT: usize = (isize::MAX) as usize;

enum Count {
    Single(Cell<usize>),
    Multi(AtomicUsize),
}

struct Inner<T: ?Sized> {
    count: UnsafeCell<Count>,
    data: T,
}

impl<T> Inner<T> {
    fn new(data: T) -> Box<Self> {
        Box::new(Self {
            count: Count::Single(1.into()).into(),
            data,
        })
    }
}

impl<T: ?Sized> Inner<T> {
    unsafe fn make_multi_threaded(&self) {
        let count = match &*self.count.get() {
            Count::Single(cell) => cell.get(),
            Count::Multi(_) => return,
        };
        // We're single-threaded, so we can safely do an unsynchronized write.
        *self.count.get() = Count::Multi(count.into());
    }

    unsafe fn make_single_threaded(&self) -> bool {
        let count = match &*self.count.get() {
            Count::Single(_) => return true,
            Count::Multi(atom) => atom.load(Ordering::SeqCst),
        };
        if count == 1 {
            // We're the sole owner, so we can safely do an unsynchronized write.
            *self.count.get() = Count::Single(count.into());
            true
        } else {
            false
        }
    }

    fn increment(&self) -> usize {
        unsafe {
            let count = match &*self.count.get() {
                Count::Single(cell) => {
                    let count = cell.get() + 1;
                    cell.set(count);
                    count
                }
                Count::Multi(atom) => atom.fetch_add(1, Ordering::Relaxed) + 1,
            };
            if count > MAX_REFCOUNT {
                abort();
            }
            count
        }
    }

    fn decrement(&self) -> usize {
        unsafe {
            match &*self.count.get() {
                Count::Single(cell) => {
                    let count = cell.get() - 1;
                    cell.set(count);
                    count
                }
                Count::Multi(atom) => {
                    let count = atom.fetch_sub(1, Ordering::Release) - 1;
                    if count == 0 {
                        atomic::fence(Ordering::Acquire);
                    }
                    count
                }
            }
        }
    }
}

/// A reference-counted pointer. 'Rc' stands for 'Reference Counted'.
///
/// This may or may not use atomic access for the reference count, depending on whether it is ever
/// converted to an `Arc`.
pub struct Rc<T: ?Sized> {
    inner: NonNull<Inner<T>>,
    phantom: PhantomData<T>,
}

impl<T: ?Sized> Rc<T> {
    fn inner(&self) -> &Inner<T> {
        unsafe { self.inner.as_ref() }
    }
}

impl<T> Rc<T> {
    /// Constructs a new `Rc<T>`.
    ///
    /// This is initially single-threaded, so updates to the reference count will use non-atomic
    /// access. If an `Arc` is ever created from this instance, this will cause *all* of its
    /// references to start using atomic access to the reference count.
    pub fn new(data: T) -> Self {
        Self {
            // FIXME: use `Box::into_raw_non_null` when stable
            inner: unsafe { NonNull::new_unchecked(Box::into_raw(Inner::new(data))) },
            phantom: PhantomData,
        }
    }

    /// Converts an `Arc<T>` to `Rc<T>`. This does not change its atomic property.
    pub fn from_arc(arc: Arc<T>) -> Self {
        arc.inner
    }

    /// Attempts to convert this to an unsynchronized pointer, no longer atomic. Returns `true` if
    /// successful, or `false` if there are still potentially references on other threads.
    pub fn unshare(this: &Self) -> bool {
        unsafe { this.inner().make_single_threaded() }
    }
}

impl<T: ?Sized> Clone for Rc<T> {
    fn clone(&self) -> Self {
        self.inner().increment();
        Self { ..*self }
    }
}

impl<T: ?Sized> Drop for Rc<T> {
    fn drop(&mut self) {
        if self.inner().decrement() == 0 {
            drop(unsafe { Box::from_raw(self.inner.as_ptr()) });
        }
    }
}

impl<T: ?Sized> Deref for Rc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner().data
    }
}

impl<T> From<T> for Rc<T> {
    fn from(data: T) -> Self {
        Rc::new(data)
    }
}

/// A thread-safe reference-counting pointer. 'Arc' stands for 'Atomically Reference Counted'.
#[derive(Clone)]
pub struct Arc<T: ?Sized> {
    inner: Rc<T>,
}

// NB: the inner count **must** be the synchronized `Count::Multi`!
unsafe impl<T: Send + Sync + ?Sized> Send for Arc<T> {}
unsafe impl<T: Send + Sync + ?Sized> Sync for Arc<T> {}

impl<T> Arc<T> {
    /// Constructs a new `Arc<T>`.
    pub fn new(data: T) -> Self {
        Arc::from_rc(Rc::new(data))
    }

    /// Converts an `Rc<T>` to `Arc<T>`. This changes its count to start using atomic access, even
    /// in other outstanding `Rc<T>` references to the same underlying object.
    pub fn from_rc(rc: Rc<T>) -> Self {
        unsafe { rc.inner().make_multi_threaded() };
        Self { inner: rc }
    }
}

impl<T: ?Sized> Deref for Arc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.inner
    }
}

impl<T> From<T> for Arc<T> {
    fn from(data: T) -> Self {
        Arc::new(data)
    }
}

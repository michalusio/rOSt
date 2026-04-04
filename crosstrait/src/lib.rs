#![no_std] // no standard library
#![feature(used_with_arg)]
#![deny(rust_2018_compatibility)]
#![deny(rust_2018_idioms)]

use core::any::{Any, TypeId};

extern crate alloc;
use alloc::{boxed::Box, rc::Rc, sync::Arc};

mod global;
pub use global::*;

mod critical_section;
use critical_section as _;

// Re-exports for `register!()` macro
#[doc(hidden)]
pub use gensym::gensym;

/// Contains all relevant casting functions for a type-trait pair
/// The casting path is (conceptually) `Any::downcast::<Type>() as Target`.
///
/// The intermediate concrete type must be consistent.
///
/// With `downcast_unchecked()` the fields would all be exactly the same code: storing
/// the new vtable in the fat pointer. There would not be a `Caster<T>` generic struct
/// and the registry would be a double map of `{type_id: {trait_id: vtable_addr}}`.
#[doc(hidden)]
#[allow(clippy::type_complexity)]
#[derive(Clone, Debug)]
pub struct Caster<T: ?Sized> {
    pub ref_: fn(&dyn Any) -> Option<&T>,
    pub mut_: fn(&mut dyn Any) -> Option<&mut T>,
    pub box_: fn(Box<dyn Any>) -> Result<Box<T>, Box<dyn Any>>,
    pub rc: fn(Rc<dyn Any>) -> Result<Rc<T>, Rc<dyn Any>>,
    pub arc: fn(Arc<dyn Any + Sync + Send>) -> Result<Arc<T>, Arc<dyn Any + Sync + Send>>,
}

/// Key-value pair of the type registry.
///
/// `TypeId::of()` is not const (https://github.com/rust-lang/rust/issues/77125)
/// Hence we store a maker fn and call lazily.
/// Once it is const, remove the `fn() ->` and `||` and investigate phf.
/// The caster is a static ref to a trait object since its type
/// varies with the target trait.
#[doc(hidden)]
pub type Entry<'a> = (fn() -> [TypeId; 2], &'a (dyn Any + Send + Sync));

/// Build an entry for the type-trait registry
#[macro_export]
macro_rules! entry {
    ( $ty:ty => $tr:ty $(, $flag:ident)? ) => {
        (
            (|| [::core::any::TypeId::of::<$tr>(), ::core::any::TypeId::of::<$ty>()]) as _,
            &$crate::caster!( $ty => $tr $(, $flag)? ) as _,
        )
    };
}

/// Build a `Caster` for a given concrete type and target trait
#[macro_export]
#[doc(hidden)]
macro_rules! caster {
    ( $ty:ty => $tr:ty $(, $flag:ident)? ) => {
        $crate::Caster::<$tr> {
            ref_: |any| any.downcast_ref::<$ty>().map(|t| t as _),
            mut_: |any| any.downcast_mut::<$ty>().map(|t| t as _),
            box_: |any| any.downcast::<$ty>().map(|t| t as _),
            rc: |any| any.downcast::<$ty>().map(|t| t as _),
            arc: $crate::caster!( $ty $(, $flag)? ),
        }
    };
    ( $ty:ty ) => {
        |any| any.downcast::<$ty>().map(|t| t as _)
    };
    ( $ty:ty, no_arc ) => {
        |any| Err(any)
    };
}

/// The type-trait registry
#[derive(Default, Debug, Clone)]
pub struct Registry<'a>(
    // TODO: fixed size map, ~6kB RAM
    // Note that each key is size_of([TypeId; 2]) = 32 bytes.
    // Maybe risk collisions and reduce key size, reduce hash size, or use LinearMap
    heapless::index_map::FnvIndexMap<[TypeId; 2], &'a (dyn Any + Send + Sync), { 1 << 7 }>,
);

impl<'a> Registry<'a> {
    /// Build a new registry from an iterator over key (`fn() -> [TypeId; 2]`)
    /// value (`&'a dyn Any + Send + Sync`) pairs.
    pub fn new<'b>(it: impl IntoIterator<Item = &'b Entry<'a>>) -> Self
    where
        'a: 'b,
    {
        Self(
            it.into_iter()
                .map(|(key, value)| ((*key)(), *value))
                .collect(),
        )
    }

    /// Obtain a `Caster` given the TypeId for the concrete type and a target trait `T`
    fn caster<T: ?Sized + 'static>(&self, any: TypeId) -> Option<&Caster<T>> {
        self.0.get(&[TypeId::of::<T>(), any])?.downcast_ref()
    }

    /// Whether the `Any` can be cast to the target trait T
    pub fn castable_ref<T: ?Sized + 'static>(&self, any: &dyn Any) -> bool {
        self.0.contains_key(&[TypeId::of::<T>(), any.type_id()])
    }

    /// Whether the concrete type U can be case to the target trait T
    pub fn castable<T: ?Sized + 'static, U: ?Sized + 'static>(&self) -> bool {
        self.0.contains_key(&[TypeId::of::<T>(), TypeId::of::<U>()])
    }

    /// Cast to an immuatbly borrowed trait object
    pub fn cast_ref<'b, T: ?Sized + 'static>(&self, any: &'b dyn Any) -> Option<&'b T> {
        (self.caster(any.type_id())?.ref_)(any)
    }

    /// Cast to a mutably borrowed trait object
    pub fn cast_mut<'b, T: ?Sized + 'static>(&self, any: &'b mut dyn Any) -> Option<&'b mut T> {
        (self.caster((*any).type_id())?.mut_)(any)
    }

    /// Cast to a boxed trait object
    pub fn cast_box<T: ?Sized + 'static>(&self, any: Box<dyn Any>) -> Result<Box<T>, Box<dyn Any>> {
        match self.caster((*any).type_id()) {
            Some(c) => (c.box_)(any),
            None => Err(any),
        }
    }

    /// Cast to a ref counted trait object
    pub fn cast_rc<T: ?Sized + 'static>(&self, any: Rc<dyn Any>) -> Result<Rc<T>, Rc<dyn Any>> {
        match self.caster((*any).type_id()) {
            Some(c) => (c.rc)(any),
            None => Err(any),
        }
    }

    /// Cast to an atomically ref counted trait object
    pub fn cast_arc<T: ?Sized + 'static>(
        &self,
        any: Arc<dyn Any + Sync + Send>,
    ) -> Result<Arc<T>, Arc<dyn Any + Sync + Send>> {
        match self.caster((*any).type_id()) {
            Some(c) => (c.arc)(any),
            None => Err(any),
        }
    }
}

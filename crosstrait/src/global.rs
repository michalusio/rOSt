use core::any::Any;

extern crate alloc;
use alloc::{boxed::Box, rc::Rc, sync::Arc};

use once_cell::sync::Lazy;

#[doc(hidden)]
pub use linkme;

use crate::{Entry, Registry};

/// Static slice of key maker fns and Caster trait objects
#[doc(hidden)]
#[linkme::distributed_slice]
pub static REGISTRY_KV: [Entry<'static>];

/// Register a type and target trait in the registry
#[macro_export]
macro_rules! register {
    ( $ty:ty => $tr:ty $(, $flag:ident)? ) => {
        $crate::register! { $crate::REGISTRY_KV: $ty => $tr $(, $flag)? }
    };
    ( $reg:path: $ty:ty => $tr:ty $(, $flag:ident)? ) => {
        $crate::gensym! { $crate::register!{ $reg: $ty => $tr $(, $flag)? } }
    };
    ( $name:ident, $reg:path: $ty:ty => $tr:ty $(, $flag:ident)? ) => {
        #[$crate::linkme::distributed_slice($reg)]
        #[linkme(crate=$crate::linkme)]
        static $name: $crate::Entry<'static> = $crate::entry!( $ty => $tr $(, $flag)? );
    };
}

/// The global static type-trait registry
pub static REGISTRY: Lazy<Registry<'static>> = Lazy::new(|| Registry::new(REGISTRY_KV));

/// Whether a `dyn Any` can be cast to a given trait object
pub trait CastableRef {
    /// Whether we can be cast to a given trait object
    fn castable<T: ?Sized + 'static>(self) -> bool;
}

impl CastableRef for &dyn Any {
    fn castable<T: ?Sized + 'static>(self) -> bool {
        REGISTRY.castable_ref::<T>(self)
    }
}

/// Whether this concrete type can be cast to a given trait object
pub trait Castable {
    /// Whether this type is castable to the given trait object
    fn castable<T: ?Sized + 'static>() -> bool;
}

impl<U: ?Sized + 'static> Castable for U {
    fn castable<T: ?Sized + 'static>() -> bool {
        REGISTRY.castable::<T, U>()
    }
}

/// Cast an `dyn Any` to another given trait object
///
/// Uses the global type-trait registry.
pub trait Cast<T> {
    /// Cast a `dyn Any` (reference or smart pointer) to a given trait object
    fn cast(self) -> Option<T>;
}

impl<'b, T: ?Sized + 'static> Cast<&'b T> for &'b dyn Any {
    fn cast(self) -> Option<&'b T> {
        REGISTRY.cast_ref(self)
    }
}

impl<'b, T: ?Sized + 'static> Cast<&'b mut T> for &'b mut dyn Any {
    fn cast(self) -> Option<&'b mut T> {
        REGISTRY.cast_mut(self)
    }
}

impl<T: ?Sized + 'static> Cast<Box<T>> for Box<dyn Any> {
    fn cast(self) -> Option<Box<T>> {
        REGISTRY.cast_box(self).ok()
    }
}

impl<T: ?Sized + 'static> Cast<Rc<T>> for Rc<dyn Any> {
    fn cast(self) -> Option<Rc<T>> {
        REGISTRY.cast_rc(self).ok()
    }
}

impl<T: ?Sized + 'static> Cast<Arc<T>> for Arc<dyn Any + Sync + Send> {
    fn cast(self) -> Option<Arc<T>> {
        REGISTRY.cast_arc(self).ok()
    }
}

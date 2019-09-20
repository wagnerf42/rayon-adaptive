//! Boolean structs and traits for implementing OR in where clauses.
//! For example, let's say we have three traits `Foo` `Bar` and `Baz` and
//! we want to implement `Baz` for all types which implement `Foo` OR `Baz`.
//! This is **NOT** possible:
//! ```compile_fail
//! trait Foo {}
//! trait Bar {}
//! trait Baz {}
//! impl<I: Foo> Baz for I {}
//! impl<I: Bar> Baz for I {}
//! ```
//! because we might have two conflicting implementations in case I implements both `Foo` and
//! `Bar`.
//!
//! Now, using the macros in this module you can do:
//! ```
//! use rayon_adaptive::{marked, or_trait};
//! trait Foo {}
//! trait Bar {}
//! trait Baz {}
//! marked!(Foo, ImplementsFoo);
//! marked!(Bar, ImplementsBar);
//! or_trait!(ImplementsFoo, ImplementsBar, FooOrBar);
//! impl<I: FooOrBar> Baz for I {}
//! ```
//!
//! The downside is that you need to explicitely declare when a type will NOT implement a *marked*
//! trait. For example if `u32` implements `Foo` but not `Bar` you must do:
//! ```
//! use rayon_adaptive::{marked, or_trait, deny_implementation};
//! trait Foo {}
//! trait Bar {}
//! trait Baz {}
//! marked!(Foo, ImplementsFoo);
//! marked!(Bar, ImplementsBar);
//! or_trait!(ImplementsFoo, ImplementsBar, FooOrBar);
//! impl<I: FooOrBar> Baz for I {}
//! impl Foo for u32 {}
//! deny_implementation!(ImplementsBar, u32);
//! ```

//TODO: I want it all private but still want the doctest ?
pub struct TrueType;
pub struct FalseType;

pub trait TraitTrue {}
impl TraitTrue for TrueType {}

pub trait Or<B> {
    type Res;
}

impl<B> Or<B> for TrueType {
    type Res = TrueType;
}

impl<B> Or<B> for FalseType {
    type Res = B;
}

#[macro_export]
macro_rules! marked {
    ($trait: ident, $itrait: ident) => {
        trait $itrait {
            type R: $crate::boolean_types::Or<$crate::boolean_types::TrueType>
                + $crate::boolean_types::Or<$crate::boolean_types::FalseType>;
        }
        impl<I: $trait> $itrait for I {
            type R = $crate::boolean_types::TrueType;
        }
    };
}

#[macro_export]
macro_rules! or_trait {
    ($trait1: ident, $trait2: ident, $or_trait: ident) => {
        trait $or_trait {}
        impl<I> $or_trait for I
        where
            Self: $trait1,
            Self: $trait2,
            <Self as $trait1>::R: $crate::boolean_types::Or<<Self as $trait2>::R>,
            <<Self as $trait1>::R as $crate::boolean_types::Or<<Self as $trait2>::R>>::Res:
                $crate::boolean_types::TraitTrue,
        {
        }
    };
}

#[macro_export]
macro_rules! deny_implementation {
    ($trait: ident, $type: ty) => {
        impl $trait for $type {
            type R = $crate::boolean_types::FalseType;
        }
    };
}

#![feature(negative_impls)]
#![feature(auto_traits)]
#![feature(option_result_unwrap_unchecked)]
#![feature(never_type)]

mod internal {
    use std::any::Any;
    use std::hint::unreachable_unchecked;
    use std::marker::PhantomData;

    mod impls {
        use super::*;
        use anonymous_enums_proc_macro::invoke_with_idents;
        anonymous_enums_proc_macro::impl_contains_for_tuples! {}
        macro_rules! impl_sealed {
            ($($($ident:ident)*;)*) => {
                $(
                    impl<$($ident),*> Sealed for ($($ident,)*) {}
                )*
            }
        }
        invoke_with_idents!(impl_sealed);
        macro_rules! impl_contains_all {
            ($($($ident:ident)*;)*) => {
                $(
                    unsafe impl<Main, $($ident),*> ContainsAll<($($ident,)*)> for Main where $(Main: Contains<$ident>,)* {}
                )*
            }
        }
        invoke_with_idents!(impl_contains_all);
    }

    /// Safety: For all types `U` where T: Contains<U>, Self: Contains<U> must hold
    pub unsafe trait ContainsAll<T> {}
    pub trait Sealed {}

    /// Safety: For all types `U` excluding `T`, where `Self: Contains<U>`, `<Self as Contains<T>>::Without: Contains<U>` must hold
    pub unsafe trait Contains<T>: Sealed {
        type Without;
    }

    pub struct PrivatePair<T, U>(T, U);

    pub auto trait TypesNotEqual {}

    impl<T> !TypesNotEqual for PrivatePair<T, T> {}
    pub trait NotEqual {}

    impl<T, U> NotEqual for (T, U) where PrivatePair<T, U>: TypesNotEqual {}

    pub struct OneOf<T>(Box<dyn Any>, PhantomData<T>);

    impl<T> OneOf<T> {
        pub fn new<U>(x: U) -> Self
        where
            T: Contains<U>,
            U: 'static,
        {
            Self(Box::new(x), PhantomData)
        }

        pub fn take<U>(self) -> Result<U, OneOf<<T as Contains<U>>::Without>>
        where
            T: Contains<U>,
            U: 'static,
        {
            self.0
                .downcast()
                .map(|b| *b)
                .map_err(|e| OneOf(e, PhantomData))
        }

        pub fn into_inner(self) -> Box<dyn Any> {
            self.0
        }
    }

    impl<T: Empty> OneOf<T> {
        #[inline(always)]
        pub fn infallible(self) -> ! {
            self.into()
        }
    }

    impl<T, U> From<OneOf<U>> for OneOf<T>
    where
        (T, U): NotEqual,
        T: ContainsAll<U>,
    {
        fn from(s: OneOf<U>) -> OneOf<T> {
            OneOf(s.0, PhantomData)
        }
    }

    impl<T: Empty> From<OneOf<T>> for ! {
        #[inline(always)]
        fn from(_: OneOf<T>) -> Self {
            unsafe { unreachable_unchecked() }
        }
    }

    pub trait Empty {}
    impl Empty for () {}

    #[extend::ext(name = ResultExt)]
    pub impl<T, E> Result<T, OneOf<E>> {
        fn infallible(self) -> T
        where
            E: Empty,
        {
            match self {
                Ok(x) => x,
                Err(e) => e.infallible(),
            }
        }

        fn handle<U>(
            self,
            f: impl FnOnce(U) -> Result<T, OneOf<<E as Contains<U>>::Without>>,
        ) -> Result<T, OneOf<<E as Contains<U>>::Without>>
        where
            E: Contains<U>,
            U: 'static,
        {
            match self {
                Ok(t) => Ok(t),
                Err(e) => match e.take::<U>() {
                    Ok(e) => f(e),
                    Err(e) => Err(e),
                },
            }
        }
    }
}

pub use anonymous_enums_proc_macro::match_type;
pub use internal::{OneOf, ResultExt, Contains, ContainsAll};

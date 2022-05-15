/// Implements the `Writer`, `Rewrite` and `Typed` traits.
macro_rules! writer {
    ($ty:ident: |$obj:ident| $($tts:tt)*) => {
        impl<'a> Writer<'a> for $ty<'a> {
            #[inline]
            fn start($obj: Obj<'a>) -> Self {
                $($tts)*
            }
        }

        impl<'a, 'any> Rewrite<'a> for $ty<'any> {
            type Output = $ty<'a>;
        }
    };
}

/// Implements `Deref` and `DerefMut` by delegating to a field of a struct.
macro_rules! deref {
    ($a:lifetime, $from:ty => $to:ty, $field:ident) => {
        impl<$a> std::ops::Deref for $from {
            type Target = $to;

            #[inline]
            fn deref(&self) -> &Self::Target {
                &self.$field
            }
        }

        impl<$a> std::ops::DerefMut for $from {
            #[inline]
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.$field
            }
        }
    };
}

/// Implements `Deref` and `DerefMut` by delegating to a field of a struct.
macro_rules! deref {
    ($a:lifetime, $from:ty => $to:ty, $field:ident) => {
        impl<$a> std::ops::Deref for $from {
            type Target = $to;

            fn deref(&self) -> &Self::Target {
                &self.$field
            }
        }

        impl<$a> std::ops::DerefMut for $from {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.$field
            }
        }
    };
}

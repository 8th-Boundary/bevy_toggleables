use core::ops::Deref;

use crate::ToggleableComponent;

pub trait HasToggle<T: ToggleableComponent> {
    fn toggle_ref(&self) -> &T;
}

impl<T, U> HasToggle<T> for U
where
    T: ToggleableComponent,
    U: Deref<Target = T>,
{
    #[inline]
    fn toggle_ref(&self) -> &T {
        self.deref()
    }
}

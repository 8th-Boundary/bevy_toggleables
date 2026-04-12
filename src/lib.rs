//! `bevy_toggleables` provides a tiny ECS-facing contract for components that
//! can be logically enabled or disabled without changing archetypes.

extern crate self as bevy_toggleables;

use bevy_ecs::component::Component;
use bevy_ecs::component::Mutable;

mod access;

#[cfg(feature = "toggle_ext")]
mod commands;

#[cfg(feature = "toggle_ext")]
mod iter;

pub mod prelude;

/// A Bevy component with an in-place boolean toggle.
///
/// The toggle state lives inside the component itself, so changing it does not
/// move the entity between archetypes.
pub trait ToggleableComponent: Component<Mutability = Mutable> {
    /// Returns whether this component is logically active.
    fn is_active(&self) -> bool;

    /// Sets whether this component is logically active.
    fn set_active(&mut self, active: bool);

    /// Toggles the active state in place.
    #[inline]
    fn toggle(&mut self) {
        self.set_active(!self.is_active());
    }
}

#[cfg(feature = "derive")]
pub use bevy_toggleables_derive::Toggleable;

#[cfg(feature = "toggle_ext")]
pub use commands::ToggleEntityCommandsExt;

#[cfg(feature = "toggle_ext")]
pub use iter::ToggleIterExt;

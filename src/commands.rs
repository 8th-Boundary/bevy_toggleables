use bevy_ecs::system::EntityCommands;
use bevy_ecs::world::EntityWorldMut;

use crate::ToggleableComponent;

pub trait ToggleEntityCommandsExt {
    fn set_toggle_if_present<T: ToggleableComponent>(&mut self, active: bool) -> &mut Self;
    fn toggle_if_present<T: ToggleableComponent>(&mut self) -> &mut Self;
}

impl ToggleEntityCommandsExt for EntityCommands<'_> {
    #[inline]
    fn set_toggle_if_present<T: ToggleableComponent>(&mut self, active: bool) -> &mut Self {
        self.queue_silenced(move |mut entity: EntityWorldMut| {
            if let Some(mut component) = entity.get_mut::<T>() {
                component.set_active(active);
            }
        });
        self
    }

    #[inline]
    fn toggle_if_present<T: ToggleableComponent>(&mut self) -> &mut Self {
        self.queue_silenced(|mut entity: EntityWorldMut| {
            if let Some(mut component) = entity.get_mut::<T>() {
                component.toggle();
            }
        });
        self
    }
}

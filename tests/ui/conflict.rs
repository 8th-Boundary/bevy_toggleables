use bevy_ecs::prelude::*;
use bevy_toggleables::prelude::*;

#[derive(Component, Toggleable)]
struct Conflict {
    #[toggleable]
    enabled: bool,
    active: bool,
}

fn main() {}

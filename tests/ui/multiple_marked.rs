use bevy_ecs::prelude::*;
use bevy_toggleables::prelude::*;

#[derive(Component, Toggleable)]
struct MultipleMarked {
    #[toggleable]
    first: bool,
    #[toggleable]
    second: bool,
}

fn main() {}

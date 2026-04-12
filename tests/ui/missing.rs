use bevy_ecs::prelude::*;
use bevy_toggleables::prelude::*;

#[derive(Component, Toggleable)]
struct Missing {
    stacks: u8,
}

fn main() {}

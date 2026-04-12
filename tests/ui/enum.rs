use bevy_ecs::prelude::*;
use bevy_toggleables::prelude::*;

#[derive(Component, Toggleable)]
enum BadEnum {
    A,
}

fn main() {}

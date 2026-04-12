use bevy_ecs::prelude::*;
use bevy_toggleables::prelude::*;

#[derive(Component, Toggleable)]
struct BadAttr {
    #[toggleable(foo)]
    enabled: bool,
}

fn main() {}

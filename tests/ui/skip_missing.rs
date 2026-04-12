use bevy_ecs::prelude::*;
use bevy_toggleables::prelude::*;

#[derive(Component)]
struct Label;

#[derive(Component, Toggleable)]
struct Frozen {
    active: bool,
}

fn system(query: Query<&Label>) {
    let _ = query.iter().skip_inactive::<Frozen>();
}

fn main() {}

use bevy_ecs::prelude::*;
use bevy_toggleables::prelude::*;

#[derive(Component, Toggleable)]
struct Frozen {
    active: bool,
}

#[derive(Component, Toggleable)]
struct Burning {
    active: bool,
}

#[derive(Component)]
struct Label;

fn system(query: Query<(&Frozen, &Label)>) {
    let _ = query.iter().where_toggles::<(Frozen, Burning)>((true, false));
}

fn main() {}

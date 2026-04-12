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

fn system(query: Query<(&Frozen, &Burning)>) {
    let _ = query.iter().where_toggles::<(Burning, Frozen)>((true, false));
}

fn main() {}

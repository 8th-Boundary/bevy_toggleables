use bevy_ecs::prelude::*;
use bevy_toggleables::prelude::*;

#[derive(Component, Toggleable, Debug)]
#[allow(dead_code)]
struct Frozen {
    active: bool,
    stacks: u8,
}

#[derive(Component, Toggleable, Debug)]
#[allow(dead_code)]
struct Burning {
    #[toggleable]
    enabled: bool,
}

fn main() {
    let mut world = World::default();
    world.spawn((
        Frozen {
            active: true,
            stacks: 3,
        },
        Burning { enabled: false },
    ));

    // Direct mutation is the hot path.
    {
        let mut query = world.query::<&mut Frozen>();
        for mut frozen in query.iter_mut(&mut world) {
            frozen.toggle();
        }
    }

    // Filtered iteration stays zero-allocation.
    {
        let mut query = world.query::<(&Frozen, &Burning)>();
        let active = query
            .iter(&world)
            .where_toggles::<(Frozen, Burning)>((false, false))
            .count();

        assert_eq!(active, 1);
    }

    {
        let mut query = world.query::<&Frozen>();
        let active = query.iter(&world).skip_inactive::<Frozen>().count();

        assert_eq!(active, 0);
    }
}

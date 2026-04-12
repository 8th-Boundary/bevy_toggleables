use bevy_ecs::prelude::*;
use bevy_ecs::system::RunSystemOnce;
use bevy_toggleables::prelude::*;

#[derive(Component, Toggleable, Debug)]
#[allow(dead_code)]
struct Frozen {
    active: bool,
    stacks: u8,
}

#[derive(Component, Toggleable, Debug)]
struct Burning {
    #[toggleable]
    enabled: bool,
}

#[derive(Component, Toggleable, Debug)]
#[component(storage = "SparseSet")]
#[allow(dead_code)]
struct Ember {
    #[toggleable]
    active: bool,
    heat: u32,
}

#[derive(Component, Toggleable, Debug)]
#[allow(dead_code)]
struct Shield<T: Send + Sync + 'static> {
    #[toggleable]
    enabled: bool,
    payload: T,
}

#[derive(Component, Debug)]
struct Label(&'static str);

#[test]
fn derive_methods_follow_the_selected_field() {
    let mut frozen = Frozen {
        active: true,
        stacks: 3,
    };

    assert!(frozen.is_active());
    frozen.toggle();
    assert!(!frozen.is_active());
    frozen.set_active(true);
    assert!(frozen.is_active());
}

#[test]
fn derive_supports_sparse_storage() {
    let mut ember = Ember {
        active: true,
        heat: 99,
    };

    assert!(ember.is_active());
    ember.toggle();
    assert!(!ember.is_active());
    ember.set_active(true);
    assert!(ember.is_active());
    assert_eq!(ember.heat, 99);
}

#[test]
fn derive_supports_generic_structs_with_explicit_toggle_fields() {
    let mut shield = Shield {
        enabled: true,
        payload: 7u32,
    };

    assert!(shield.is_active());
    shield.toggle();
    assert!(!shield.is_active());
    shield.set_active(true);
    assert!(shield.is_active());
    assert_eq!(shield.payload, 7);
}

#[test]
fn entity_commands_toggle_only_when_component_exists() {
    let mut world = World::default();
    let keep = world
        .spawn((
            Frozen {
                active: true,
                stacks: 1,
            },
            Burning { enabled: true },
        ))
        .id();
    let missing = world.spawn_empty().id();

    world
        .run_system_once(move |mut commands: Commands| {
            commands.entity(keep).set_toggle_if_present::<Frozen>(false);
            commands.entity(keep).toggle_if_present::<Frozen>();
            commands
                .entity(missing)
                .set_toggle_if_present::<Frozen>(true);
            commands.entity(missing).toggle_if_present::<Frozen>();
        })
        .unwrap();

    assert!(world.get::<Frozen>(keep).unwrap().is_active());
    assert!(world.get::<Frozen>(missing).is_none());
}

#[test]
fn skip_inactive_filters_single_component_queries() {
    let mut world = World::default();
    world.spawn(Frozen {
        active: true,
        stacks: 1,
    });
    world.spawn(Frozen {
        active: false,
        stacks: 2,
    });
    world.spawn(Frozen {
        active: true,
        stacks: 3,
    });

    let count = world
        .run_system_once(|query: Query<&Frozen>| query.iter().skip_inactive::<Frozen>().count())
        .unwrap();

    assert_eq!(count, 2);
}

#[test]
fn skip_active_filters_single_component_queries() {
    let mut world = World::default();
    world.spawn(Frozen {
        active: true,
        stacks: 1,
    });
    world.spawn(Frozen {
        active: false,
        stacks: 2,
    });
    world.spawn(Frozen {
        active: false,
        stacks: 3,
    });

    let count = world
        .run_system_once(|query: Query<&Frozen>| query.iter().skip_active::<Frozen>().count())
        .unwrap();

    assert_eq!(count, 2);
}

#[test]
fn where_toggles_filters_tuple_prefix_queries() {
    let mut world = World::default();
    world.spawn((
        Frozen {
            active: true,
            stacks: 1,
        },
        Burning { enabled: false },
        Label("match"),
    ));
    world.spawn((
        Frozen {
            active: true,
            stacks: 2,
        },
        Burning { enabled: true },
        Label("skip-burning"),
    ));
    world.spawn((
        Frozen {
            active: false,
            stacks: 3,
        },
        Burning { enabled: false },
        Label("skip-frozen"),
    ));

    let labels = world
        .run_system_once(|query: Query<(&Frozen, &Burning, &Label)>| {
            query
                .iter()
                .where_toggles::<(Frozen, Burning)>((true, false))
                .map(|(_, _, label)| label.0)
                .collect::<Vec<_>>()
        })
        .unwrap();

    assert_eq!(labels, vec!["match"]);
}

#[test]
fn where_toggles_works_with_iter_mut() {
    let mut world = World::default();
    let target = world
        .spawn((
            Frozen {
                active: true,
                stacks: 1,
            },
            Burning { enabled: false },
        ))
        .id();
    world.spawn((
        Frozen {
            active: false,
            stacks: 2,
        },
        Burning { enabled: false },
    ));

    world
        .run_system_once(|mut query: Query<(&mut Frozen, &Burning)>| {
            for (mut frozen, burning) in query
                .iter_mut()
                .where_toggles::<(Frozen, Burning)>((true, false))
            {
                assert!(!burning.is_active());
                frozen.set_active(false);
            }
        })
        .unwrap();

    assert!(!world.get::<Frozen>(target).unwrap().is_active());
}

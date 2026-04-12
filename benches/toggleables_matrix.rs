use std::{collections::HashMap, env, time::Duration};

use bevy_ecs::prelude::*;
use bevy_toggleables::prelude::*;
use criterion::{black_box, BatchSize, BenchmarkId, Criterion, Throughput};

const DEFAULT_ENTITY_COUNT: usize = 5_000;
const WARM_UP_TIME: Duration = Duration::from_millis(200);
const MEASUREMENT_TIME: Duration = Duration::from_millis(500);
const SAMPLE_SIZE: usize = 20;

#[derive(Component, Clone)]
#[allow(dead_code)]
struct TableValue {
    value: u32,
}

#[derive(Component, Clone)]
#[component(storage = "SparseSet")]
#[allow(dead_code)]
struct SparseValue {
    value: u32,
}

#[derive(Clone)]
#[allow(dead_code)]
struct MapValue {
    value: u32,
}

#[derive(Resource, Default)]
struct EntityMap(HashMap<Entity, MapValue>);

#[derive(Component, Toggleable, Clone)]
#[allow(dead_code)]
struct ToggleTableValue {
    active: bool,
    value: u32,
}

#[derive(Component, Toggleable, Clone)]
#[component(storage = "SparseSet")]
#[allow(dead_code)]
struct ToggleSparseValue {
    #[toggleable]
    enabled: bool,
    value: u32,
}

fn main() {
    let entity_count = parse_entity_count();
    let mut criterion = Criterion::default()
        .warm_up_time(WARM_UP_TIME)
        .measurement_time(MEASUREMENT_TIME)
        .sample_size(SAMPLE_SIZE);

    bench_add(&mut criterion, entity_count);
    bench_modify(&mut criterion, entity_count);
    bench_remove(&mut criterion, entity_count);
    bench_rapid(&mut criterion, entity_count);

    criterion.final_summary();
}

fn parse_entity_count() -> usize {
    let mut entity_count = DEFAULT_ENTITY_COUNT;
    let mut args = env::args().skip(1);

    while let Some(arg) = args.next() {
        if matches!(arg.as_str(), "--entities" | "--entity-count" | "-e") {
            let flag = arg;
            let value = args
                .next()
                .unwrap_or_else(|| panic!("expected a value after `{flag}`"));
            entity_count = parse_entity_count_value(&value);
        } else if let Some(value) = arg.strip_prefix("--entities=") {
            entity_count = parse_entity_count_value(value);
        } else if let Some(value) = arg.strip_prefix("--entity-count=") {
            entity_count = parse_entity_count_value(value);
        }
    }

    assert!(entity_count > 0, "entity count must be greater than zero");
    entity_count
}

fn parse_entity_count_value(value: &str) -> usize {
    value
        .parse::<usize>()
        .unwrap_or_else(|error| panic!("invalid entity count `{value}`: {error}"))
}

fn spawn_empty_entities(world: &mut World, entity_count: usize) -> Vec<Entity> {
    let mut entities = Vec::with_capacity(entity_count);
    for _ in 0..entity_count {
        entities.push(world.spawn_empty().id());
    }
    entities
}

fn spawn_component_entities<C, F>(
    world: &mut World,
    entity_count: usize,
    mut make: F,
) -> Vec<Entity>
where
    C: Component,
    F: FnMut(usize) -> C,
{
    let mut entities = Vec::with_capacity(entity_count);
    for index in 0..entity_count {
        entities.push(world.spawn(make(index)).id());
    }
    entities
}

fn build_empty_world(entity_count: usize) -> (World, Vec<Entity>) {
    let mut world = World::default();
    let entities = spawn_empty_entities(&mut world, entity_count);
    (world, entities)
}

fn build_component_world<C, F>(entity_count: usize, make: F) -> (World, Vec<Entity>)
where
    C: Component,
    F: FnMut(usize) -> C,
{
    let mut world = World::default();
    let entities = spawn_component_entities(&mut world, entity_count, make);
    (world, entities)
}

fn build_map_world(entity_count: usize, populate: bool) -> (World, Vec<Entity>) {
    let mut world = World::default();
    world.insert_resource(EntityMap(HashMap::with_capacity(entity_count)));
    let entities = spawn_empty_entities(&mut world, entity_count);

    if populate {
        let mut map = world.resource_mut::<EntityMap>();
        for (index, entity) in entities.iter().copied().enumerate() {
            map.0.insert(
                entity,
                MapValue {
                    value: index as u32,
                },
            );
        }
    }

    (world, entities)
}

macro_rules! bench_case {
    ($group:expr, $name:expr, $entity_count:expr, $setup:expr, $work:expr $(,)?) => {{
        $group.bench_function(BenchmarkId::new($name, $entity_count), |b| {
            b.iter_batched($setup, $work, BatchSize::SmallInput)
        });
    }};
}

fn bench_add(c: &mut Criterion, entity_count: usize) {
    let mut group = c.benchmark_group("add");
    group.throughput(Throughput::Elements(entity_count as u64));

    bench_case!(
        &mut group,
        "normal_table_insert",
        entity_count,
        || build_empty_world(entity_count),
        |(mut world, entities)| {
            let mut sink = 0u64;
            for (index, entity) in entities.into_iter().enumerate() {
                world.entity_mut(entity).insert(TableValue {
                    value: index as u32,
                });
                sink = sink.wrapping_add(index as u64);
            }
            black_box(sink);
        },
    );

    bench_case!(
        &mut group,
        "normal_sparse_insert",
        entity_count,
        || build_empty_world(entity_count),
        |(mut world, entities)| {
            let mut sink = 0u64;
            for (index, entity) in entities.into_iter().enumerate() {
                world.entity_mut(entity).insert(SparseValue {
                    value: index as u32,
                });
                sink = sink.wrapping_add(index as u64);
            }
            black_box(sink);
        },
    );

    bench_case!(
        &mut group,
        "entity_map_insert",
        entity_count,
        || build_map_world(entity_count, false),
        |(mut world, entities)| {
            let mut sink = 0u64;
            let mut map = world.resource_mut::<EntityMap>();
            for (index, entity) in entities.into_iter().enumerate() {
                map.0.insert(
                    entity,
                    MapValue {
                        value: index as u32,
                    },
                );
                sink = sink.wrapping_add(index as u64);
            }
            black_box(sink);
        },
    );

    bench_case!(
        &mut group,
        "toggle_table_insert_missing",
        entity_count,
        || build_empty_world(entity_count),
        |(mut world, entities)| {
            let mut sink = 0u64;
            for (index, entity) in entities.into_iter().enumerate() {
                world.entity_mut(entity).insert(ToggleTableValue {
                    active: true,
                    value: index as u32,
                });
                sink = sink.wrapping_add(index as u64);
            }
            black_box(sink);
        },
    );

    bench_case!(
        &mut group,
        "toggle_table_enable_disabled",
        entity_count,
        || {
            build_component_world(entity_count, |index| ToggleTableValue {
                active: false,
                value: index as u32,
            })
        },
        |(mut world, _entities)| {
            let mut sink = 0u64;
            let mut query = world.query::<&mut ToggleTableValue>();
            for mut component in query.iter_mut(&mut world) {
                component.set_active(true);
                sink = sink.wrapping_add(component.value as u64);
            }
            black_box(sink);
        },
    );

    bench_case!(
        &mut group,
        "toggle_sparse_insert_missing",
        entity_count,
        || build_empty_world(entity_count),
        |(mut world, entities)| {
            let mut sink = 0u64;
            for (index, entity) in entities.into_iter().enumerate() {
                world.entity_mut(entity).insert(ToggleSparseValue {
                    enabled: true,
                    value: index as u32,
                });
                sink = sink.wrapping_add(index as u64);
            }
            black_box(sink);
        },
    );

    bench_case!(
        &mut group,
        "toggle_sparse_enable_disabled",
        entity_count,
        || {
            build_component_world(entity_count, |index| ToggleSparseValue {
                enabled: false,
                value: index as u32,
            })
        },
        |(mut world, _entities)| {
            let mut sink = 0u64;
            let mut query = world.query::<&mut ToggleSparseValue>();
            for mut component in query.iter_mut(&mut world) {
                component.set_active(true);
                sink = sink.wrapping_add(component.value as u64);
            }
            black_box(sink);
        },
    );

    group.finish();
}

fn bench_modify(c: &mut Criterion, entity_count: usize) {
    let mut group = c.benchmark_group("modify");
    group.throughput(Throughput::Elements(entity_count as u64));

    bench_case!(
        &mut group,
        "normal_table_update",
        entity_count,
        || {
            build_component_world(entity_count, |index| TableValue {
                value: index as u32,
            })
        },
        |(mut world, _entities)| {
            let mut sink = 0u64;
            let mut query = world.query::<&mut TableValue>();
            for mut component in query.iter_mut(&mut world) {
                component.value = component.value.wrapping_add(1);
                sink = sink.wrapping_add(component.value as u64);
            }
            black_box(sink);
        },
    );

    bench_case!(
        &mut group,
        "normal_sparse_update",
        entity_count,
        || {
            build_component_world(entity_count, |index| SparseValue {
                value: index as u32,
            })
        },
        |(mut world, _entities)| {
            let mut sink = 0u64;
            let mut query = world.query::<&mut SparseValue>();
            for mut component in query.iter_mut(&mut world) {
                component.value = component.value.wrapping_add(1);
                sink = sink.wrapping_add(component.value as u64);
            }
            black_box(sink);
        },
    );

    bench_case!(
        &mut group,
        "entity_map_update",
        entity_count,
        || build_map_world(entity_count, true),
        |(mut world, entities)| {
            let mut sink = 0u64;
            let mut map = world.resource_mut::<EntityMap>();
            for entity in entities {
                let component = map.0.get_mut(&entity).expect("entry present");
                component.value = component.value.wrapping_add(1);
                sink = sink.wrapping_add(component.value as u64);
            }
            black_box(sink);
        },
    );

    bench_case!(
        &mut group,
        "toggle_table_update",
        entity_count,
        || {
            build_component_world(entity_count, |index| ToggleTableValue {
                active: true,
                value: index as u32,
            })
        },
        |(mut world, _entities)| {
            let mut sink = 0u64;
            let mut query = world.query::<&mut ToggleTableValue>();
            for mut component in query.iter_mut(&mut world) {
                component.value = component.value.wrapping_add(1);
                sink = sink.wrapping_add(component.value as u64);
            }
            black_box(sink);
        },
    );

    bench_case!(
        &mut group,
        "toggle_sparse_update",
        entity_count,
        || {
            build_component_world(entity_count, |index| ToggleSparseValue {
                enabled: true,
                value: index as u32,
            })
        },
        |(mut world, _entities)| {
            let mut sink = 0u64;
            let mut query = world.query::<&mut ToggleSparseValue>();
            for mut component in query.iter_mut(&mut world) {
                component.value = component.value.wrapping_add(1);
                sink = sink.wrapping_add(component.value as u64);
            }
            black_box(sink);
        },
    );

    group.finish();
}

fn bench_remove(c: &mut Criterion, entity_count: usize) {
    let mut group = c.benchmark_group("remove");
    group.throughput(Throughput::Elements(entity_count as u64));

    bench_case!(
        &mut group,
        "normal_table_remove",
        entity_count,
        || {
            build_component_world(entity_count, |index| TableValue {
                value: index as u32,
            })
        },
        |(mut world, entities)| {
            let mut sink = 0u64;
            for entity in entities {
                let mut entity = world.entity_mut(entity);
                let value = {
                    let component = entity.get_mut::<TableValue>().expect("entry present");
                    component.value as u64
                };
                entity.remove::<TableValue>();
                sink = sink.wrapping_add(value);
            }
            black_box(sink);
        },
    );

    bench_case!(
        &mut group,
        "normal_sparse_remove",
        entity_count,
        || {
            build_component_world(entity_count, |index| SparseValue {
                value: index as u32,
            })
        },
        |(mut world, entities)| {
            let mut sink = 0u64;
            for entity in entities {
                let mut entity = world.entity_mut(entity);
                let value = {
                    let component = entity.get_mut::<SparseValue>().expect("entry present");
                    component.value as u64
                };
                entity.remove::<SparseValue>();
                sink = sink.wrapping_add(value);
            }
            black_box(sink);
        },
    );

    bench_case!(
        &mut group,
        "entity_map_remove",
        entity_count,
        || build_map_world(entity_count, true),
        |(mut world, entities)| {
            let mut sink = 0u64;
            let mut map = world.resource_mut::<EntityMap>();
            for entity in entities {
                let value = map.0.remove(&entity).expect("entry present").value as u64;
                sink = sink.wrapping_add(value);
            }
            black_box(sink);
        },
    );

    bench_case!(
        &mut group,
        "toggle_table_remove_structural",
        entity_count,
        || {
            build_component_world(entity_count, |index| ToggleTableValue {
                active: true,
                value: index as u32,
            })
        },
        |(mut world, entities)| {
            let mut sink = 0u64;
            for entity in entities {
                let mut entity = world.entity_mut(entity);
                let value = {
                    let component = entity.get_mut::<ToggleTableValue>().expect("entry present");
                    component.value as u64
                };
                entity.remove::<ToggleTableValue>();
                sink = sink.wrapping_add(value);
            }
            black_box(sink);
        },
    );

    bench_case!(
        &mut group,
        "toggle_table_disable_logical",
        entity_count,
        || {
            build_component_world(entity_count, |index| ToggleTableValue {
                active: true,
                value: index as u32,
            })
        },
        |(mut world, _entities)| {
            let mut sink = 0u64;
            let mut query = world.query::<&mut ToggleTableValue>();
            for mut component in query.iter_mut(&mut world) {
                component.set_active(false);
                sink = sink.wrapping_add(component.value as u64);
            }
            black_box(sink);
        },
    );

    bench_case!(
        &mut group,
        "toggle_sparse_remove_structural",
        entity_count,
        || {
            build_component_world(entity_count, |index| ToggleSparseValue {
                enabled: true,
                value: index as u32,
            })
        },
        |(mut world, entities)| {
            let mut sink = 0u64;
            for entity in entities {
                let mut entity = world.entity_mut(entity);
                let value = {
                    let component = entity
                        .get_mut::<ToggleSparseValue>()
                        .expect("entry present");
                    component.value as u64
                };
                entity.remove::<ToggleSparseValue>();
                sink = sink.wrapping_add(value);
            }
            black_box(sink);
        },
    );

    bench_case!(
        &mut group,
        "toggle_sparse_disable_logical",
        entity_count,
        || {
            build_component_world(entity_count, |index| ToggleSparseValue {
                enabled: true,
                value: index as u32,
            })
        },
        |(mut world, _entities)| {
            let mut sink = 0u64;
            let mut query = world.query::<&mut ToggleSparseValue>();
            for mut component in query.iter_mut(&mut world) {
                component.set_active(false);
                sink = sink.wrapping_add(component.value as u64);
            }
            black_box(sink);
        },
    );

    group.finish();
}

fn bench_rapid(c: &mut Criterion, entity_count: usize) {
    let mut group = c.benchmark_group("rapid");
    group.throughput(Throughput::Elements(
        (entity_count as u64).saturating_mul(3),
    ));

    bench_case!(
        &mut group,
        "normal_table_cycle",
        entity_count,
        || build_empty_world(entity_count),
        |(mut world, entities)| {
            let mut sink = 0u64;
            for (index, entity) in entities.into_iter().enumerate() {
                let mut entity = world.entity_mut(entity);
                entity.insert(TableValue {
                    value: index as u32,
                });
                let value = {
                    let mut component = entity.get_mut::<TableValue>().expect("entry inserted");
                    component.value = component.value.wrapping_add(1);
                    component.value as u64
                };
                entity.remove::<TableValue>();
                sink = sink.wrapping_add(value);
            }
            black_box(sink);
        },
    );

    bench_case!(
        &mut group,
        "normal_sparse_cycle",
        entity_count,
        || build_empty_world(entity_count),
        |(mut world, entities)| {
            let mut sink = 0u64;
            for (index, entity) in entities.into_iter().enumerate() {
                let mut entity = world.entity_mut(entity);
                entity.insert(SparseValue {
                    value: index as u32,
                });
                let value = {
                    let mut component = entity.get_mut::<SparseValue>().expect("entry inserted");
                    component.value = component.value.wrapping_add(1);
                    component.value as u64
                };
                entity.remove::<SparseValue>();
                sink = sink.wrapping_add(value);
            }
            black_box(sink);
        },
    );

    bench_case!(
        &mut group,
        "entity_map_cycle",
        entity_count,
        || build_map_world(entity_count, false),
        |(mut world, entities)| {
            let mut sink = 0u64;
            let mut map = world.resource_mut::<EntityMap>();
            for (index, entity) in entities.into_iter().enumerate() {
                map.0.insert(
                    entity,
                    MapValue {
                        value: index as u32,
                    },
                );
                let value = {
                    let component = map.0.get_mut(&entity).expect("entry inserted");
                    component.value = component.value.wrapping_add(1);
                    component.value as u64
                };
                map.0.remove(&entity);
                sink = sink.wrapping_add(value);
            }
            black_box(sink);
        },
    );

    bench_case!(
        &mut group,
        "toggle_table_cycle_structural",
        entity_count,
        || build_empty_world(entity_count),
        |(mut world, entities)| {
            let mut sink = 0u64;
            for (index, entity) in entities.into_iter().enumerate() {
                let mut entity = world.entity_mut(entity);
                entity.insert(ToggleTableValue {
                    active: true,
                    value: index as u32,
                });
                let value = {
                    let mut component = entity
                        .get_mut::<ToggleTableValue>()
                        .expect("entry inserted");
                    component.value = component.value.wrapping_add(1);
                    component.value as u64
                };
                entity.remove::<ToggleTableValue>();
                sink = sink.wrapping_add(value);
            }
            black_box(sink);
        },
    );

    bench_case!(
        &mut group,
        "toggle_table_cycle_logical",
        entity_count,
        || {
            build_component_world(entity_count, |index| ToggleTableValue {
                active: false,
                value: index as u32,
            })
        },
        |(mut world, _entities)| {
            let mut sink = 0u64;
            let mut query = world.query::<&mut ToggleTableValue>();
            for mut component in query.iter_mut(&mut world) {
                component.set_active(true);
                component.value = component.value.wrapping_add(1);
                component.set_active(false);
                sink = sink.wrapping_add(component.value as u64);
            }
            black_box(sink);
        },
    );

    bench_case!(
        &mut group,
        "toggle_sparse_cycle_structural",
        entity_count,
        || build_empty_world(entity_count),
        |(mut world, entities)| {
            let mut sink = 0u64;
            for (index, entity) in entities.into_iter().enumerate() {
                let mut entity = world.entity_mut(entity);
                entity.insert(ToggleSparseValue {
                    enabled: true,
                    value: index as u32,
                });
                let value = {
                    let mut component = entity
                        .get_mut::<ToggleSparseValue>()
                        .expect("entry inserted");
                    component.value = component.value.wrapping_add(1);
                    component.value as u64
                };
                entity.remove::<ToggleSparseValue>();
                sink = sink.wrapping_add(value);
            }
            black_box(sink);
        },
    );

    bench_case!(
        &mut group,
        "toggle_sparse_cycle_logical",
        entity_count,
        || {
            build_component_world(entity_count, |index| ToggleSparseValue {
                enabled: false,
                value: index as u32,
            })
        },
        |(mut world, _entities)| {
            let mut sink = 0u64;
            let mut query = world.query::<&mut ToggleSparseValue>();
            for mut component in query.iter_mut(&mut world) {
                component.set_active(true);
                component.value = component.value.wrapping_add(1);
                component.set_active(false);
                sink = sink.wrapping_add(component.value as u64);
            }
            black_box(sink);
        },
    );

    group.finish();
}

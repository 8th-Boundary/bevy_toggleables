# bevy_toggleables

## Bevy compatibility

| bevy_toggleables | Bevy |
|---|---|
| Workspace source | 0.20.x |
| Earlier 0.1.0 source | 0.18.x |

Small Bevy ECS helpers for components that carry an in-place boolean active flag.

## Why This Exists

The original problem is simple: if a component is only "active" or "inactive", a lot of code ends up modeling that state by adding and removing components or entities.

That works, but it is the expensive path in Bevy ECS. Structural changes move entities between archetypes, which is much heavier than flipping a `bool` in place.

`bevy_toggleables` keeps the state inside the component and standardizes the pattern:

- `ToggleableComponent` for the core behavior
- `#[derive(Toggleable)]` for the common `bool` field case
- optional iterator and command helpers when you want them

The hot path is still direct `&mut T` mutation from a normal query. The crate does not make a `bool` faster than plain ECS mutation. It makes the pattern consistent, checked, and easy to reuse.

## Benchmarks

The benchmark matrix compares:

- normal Bevy components
- sparse Bevy components
- an `Entity -> value` resource map
- toggleable Bevy components
- toggleable sparse Bevy components

The full results are in [bench_summary.md](./bench_summary.md). The short version:

- pure `modify` stays best with a normal table component
- `add`, `remove`, and `rapid cycle` strongly favor the toggleable path
- that advantage holds even at large scales

Representative results from the 5k-entity run:

| Family      | table | sparse | entity map | toggle table | toggle sparse | Winner |
| ----------- | ----: | -----: | ---------: | -----------: | ------------: | ------ |
| Add         | 152.79 us | 129.02 us | 53.63 us | 7.70 us | 10.22 us | toggle table |
| Modify      | 6.97 us | 10.19 us | 98.79 us | 7.46 us | 9.51 us | table |
| Remove      | 223.90 us | 195.50 us | 104.68 us | 8.62 us | 11.63 us | toggle table |
| Rapid cycle | 339.83 us | 316.10 us | 139.84 us | 9.22 us | 12.58 us | toggle table |

At 500k entities, the same pattern holds:

| Family      | table | sparse | entity map | toggle table | toggle sparse | Winner |
| ----------- | ----: | -----: | ---------: | -----------: | ------------: | ------ |
| Add         | 18.507 ms | 17.660 ms | 12.459 ms | 1.2844 ms | 2.3796 ms | toggle table |
| Modify      | 1.2910 ms | 1.7210 ms | 44.273 ms | 1.5018 ms | 1.9655 ms | table |
| Remove      | 29.459 ms | 25.497 ms | 48.288 ms | 1.4344 ms | 2.0741 ms | toggle table |
| Rapid cycle | 42.619 ms | 36.993 ms | 44.128 ms | 1.6978 ms | 2.0071 ms | toggle table |

The toggle columns use the logical path, not structural add/remove.

## Surface

The crate is intentionally narrow:

- a component-level `ToggleableComponent` trait
- a `#[derive(Toggleable)]` macro for `bool` fields
- optional command and iterator helpers

## Usage

Direct component mutation is still the hot path:

```rust
use bevy_ecs::prelude::*;
use bevy_toggleables::prelude::*;

fn flip(mut query: Query<&mut Frozen>) {
    for mut frozen in query.iter_mut() {
        frozen.toggle();
    }
}
```

Use iterator filters when you want to select active or inactive components:

```rust
use bevy_ecs::prelude::*;
use bevy_toggleables::prelude::*;

fn count_active(query: Query<&Frozen>) -> usize {
    query.iter().skip_inactive::<Frozen>().count()
}

fn count_inactive(query: Query<&Frozen>) -> usize {
    query.iter().skip_active::<Frozen>().count()
}

fn count_selected(query: Query<(&Frozen, &Burning)>) -> usize {
    query
        .iter()
        .where_toggles::<(Frozen, Burning)>((true, false))
        .count()
}
```

Use entity commands when you want a deferred update and are fine with a no-op if the component is missing:

```rust
use bevy_ecs::prelude::*;
use bevy_toggleables::prelude::*;

fn schedule_toggle(mut commands: Commands, entity: Entity) {
    commands
        .entity(entity)
        .set_toggle_if_present::<Frozen>(false)
        .toggle_if_present::<Burning>();
}
```

## Naming Contract

Use `active: bool` when you want the default convention.

Use `#[toggleable]` when the toggle field needs a different name.

```rust
use bevy_ecs::prelude::*;
use bevy_toggleables::prelude::*;

#[derive(Component, Toggleable)]
struct Frozen {
    active: bool,
    stacks: u8,
}

#[derive(Component, Toggleable)]
struct Burning {
    #[toggleable]
    enabled: bool,
}
```

For high-volume toggling, mutate the field directly through `Query<&mut T>`.

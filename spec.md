CRATE SPEC: bevy_toggleables

PURPOSE
A tiny Bevy 0.18 crate for rapidly enabling and disabling per-component behavior
without structural ECS changes.

The core principle is simple:
- the toggle state lives inside the component itself
- toggling is an in-place field mutation
- archetypes stay stable
- unrelated toggleable component systems can still run concurrently

This crate matches Bevy's strengths:
- normal ECS queries remain the core access path
- structural changes via Commands are comparatively heavier and use deferred world mutation
- read-side filtering should be an optional iterator adaptor, not a new query model

Locked v1 API surface:
- `ToggleableComponent`
- `#[derive(Toggleable)]`
- `#[toggleable]` helper field attribute
- `ToggleEntityCommandsExt`
- `ToggleIterExt`
- `skip_inactive::<T>()`
- `skip_active::<T>()`
- `where_toggles::<(A, ...)>((...))`

The derive macro name and the helper field attribute name do not need to match.
They are separate syntactic layers.
The helper field attribute is an inert marker consumed by `Toggleable`; it is not
a standalone proc-macro attribute.

Why this crate exists:
- if you only want plain bool fields and manual checks, a style guide is enough
- this crate is for teams that want a consistent toggle contract across many components
- it adds compile-time validation, a shared trait, and reusable helper APIs
- it does not make a bool store faster than plain ECS mutation
- the hot path is still direct `&mut T` mutation

A `ToggleQuery` wrapper SystemParam may be added later if usage proves it useful,
but it is not part of the locked v1 API.

--------------------------------------------------
1. CORE MODEL
--------------------------------------------------

A toggleable component is a normal Bevy Component with one bool field designated as
the toggle field.

Default convention:
- if the struct has a field named `active: bool`, that is the toggle field
- this is the preferred default because it matches `is_active()` / `set_active()`

Explicit override:
- a field marked `#[toggleable]` is the toggle field
- this allows any valid field name
- if `#[toggleable]` is present, it takes precedence over the `active` convention

Example:

    #[derive(Component, Toggleable)]
    pub struct Frozen {
        pub active: bool,
        pub stacks: u8,
    }

Behavior:
- toggle field = true  => component is logically active
- toggle field = false => component is still present, but logically inactive

Non-goals:
- No archetype-moving insert/remove to represent toggle state
- No reuse of Bevy's entity disabling semantics
- No global bitmask/state blob that would bottleneck parallel mutation

Why:
Bevy Commands are for structural world changes and are applied via deferred exclusive world access. This crate is specifically avoiding that hot-path cost for toggling.

Performance contract:
- the intended hot path is direct `&mut T` access from a query
- `set_active` should compile down to a plain bool store
- `toggle` should compile down to one load, one invert, and one store
- iterator helpers may branch, but they must not allocate or consult runtime type maps
- command helpers are convenience APIs and are not the preferred path for thousands of toggles per frame

--------------------------------------------------
2. DESIGN GOALS
--------------------------------------------------

Primary goals:
- Very fast toggle writes
- Preserve archetype stability
- Preserve scheduler concurrency by keeping action/modifier state split across component types
- Keep normal queries usable
- Provide optional filtered iteration helpers
- Minimal API surface
- No runtime maps on hot iterator paths
- Zero heap allocation for filtering helpers
- Support very high toggle volume per frame without hidden indirection

Secondary goals:
- Nice derive UX
- Clear `EntityCommands` helpers
- Easy opt-in filtering during query iteration

--------------------------------------------------
3. NON-GOALS
--------------------------------------------------

Out of scope for v1:
- Automatic mutation of normal `Query<&T>` semantics
- Transparent integration with Bevy's built-in entity disabling
- Dynamic type->bool filter maps on the hot path
- Field injection magic
- Support for tuple structs, unit structs, or enums
- Reflection-based toggle access
- Global registries for toggle state
- Parallel query-specific APIs beyond what normal Query already supports

--------------------------------------------------
4. CRATE LAYOUT
--------------------------------------------------

Workspace:

- bevy_toggleables
  Runtime crate

- bevy_toggleables_derive
  Proc-macro crate

Optional prelude:
- bevy_toggleables::prelude::*

--------------------------------------------------
5. DERIVE API
--------------------------------------------------

User API:

    #[derive(Component, Toggleable)]
    pub struct Frozen {
        #[toggleable]
        pub enabled: bool,
        pub stacks: u8,
    }

Rules:
- Only named-field structs are supported
- Exactly one toggle field must exist
- The toggle field must be type `bool`
- A field marked `#[toggleable]` is always the toggle field
- If no field is marked, a field named `active: bool` is accepted by convention
- The field attribute only marks the field; it does not rename, inject, or wrap anything
- Missing toggle field => compile error
- Wrong type for toggle field => compile error
- Multiple toggle fields => compile error
- A marked field and an `active: bool` field in the same struct => compile error
- Tuple structs => compile error
- Unit structs => compile error
- Enums => compile error

Rationale:
- Keeps behavior explicit
- Avoids hidden field injection
- Keeps generated code predictable
- Reduces macro complexity
- Avoids stringly typed field-name configuration

Generated impl:

    impl ToggleableComponent for Frozen {
        #[inline]
        fn is_active(&self) -> bool {
            self.enabled
        }

        #[inline]
        fn set_active(&mut self, active: bool) {
            self.enabled = active;
        }
    }

--------------------------------------------------
6. RUNTIME TRAIT
--------------------------------------------------

Core trait:

    pub trait ToggleableComponent {
        fn is_active(&self) -> bool;
        fn set_active(&mut self, active: bool);

        #[inline]
        fn toggle(&mut self) {
            self.set_active(!self.is_active());
        }
    }

Notes:
- No extra associated types
- No dynamic dispatch required
- All hot methods should be inline-friendly

--------------------------------------------------
7. COMMANDS API
--------------------------------------------------

Extension trait on `EntityCommands`:

    pub trait ToggleEntityCommandsExt {
        fn set_toggle_if_present<T: ToggleableComponent>(&mut self, active: bool) -> &mut Self;
        fn toggle_if_present<T: ToggleableComponent>(&mut self) -> &mut Self;
    }

Usage:

    commands.entity(e).set_toggle_if_present::<Frozen>(true);
    commands.entity(e).set_toggle_if_present::<Frozen>(false);
    commands.entity(e).toggle_if_present::<Frozen>();

Behavior:
- If entity has T, mutate `T.active` when the command is applied
- If entity lacks T, do nothing
- These helpers never insert T
- These helpers are convenience APIs only

Important:
The command path is best-effort and deferred. The fastest path is still direct `&mut T` mutation from a query or other immediate world access.

--------------------------------------------------
8. NORMAL QUERY USAGE
--------------------------------------------------

Vanilla/default behavior stays valid:

    for (a, b, c, transform) in &q {
        if !(a.is_active() && !b.is_active() && c.is_active()) {
            continue;
        }

        // ...
    }

This is the baseline pattern.
The crate must not obscure or fight this style.

--------------------------------------------------
9. OPTIONAL FILTERED ITERATION
--------------------------------------------------

Provide ergonomic filtering helpers on iterators over query items.

Primary API:

    q.iter().where_toggles::<(A, B, C)>((true, false, true))

Example:

    for (a, b, c, transform) in q
        .iter()
        .where_toggles::<(A, B, C)>((true, false, true))
    {
        // ...
    }

Mutating example:

    for (a, b, c) in q
        .iter_mut()
        .where_toggles::<(A, B, C)>((true, false, true))
    {
        // ...
    }

Single-component convenience:

    q.iter().skip_inactive::<Frozen>()

    q.iter().skip_active::<Frozen>()

Example:

    for frozen in q.iter().skip_inactive::<Frozen>() {
        // ...
    }

Semantics:
- `where_toggles::<(A, B, C)>((...))` checks the queried item's toggle states
- the requested toggle types must occupy the leading slots of the query item tuple in the same order
- extra trailing components are ignored by the filter
- `skip_inactive::<T>()` is shorthand for `where_toggles::<(T,)>((true,))`
- `skip_active::<T>()` is shorthand for `where_toggles::<(T,)>((false,))`
- the adaptor only filters yielded items; it does not alter Bevy's Query semantics
- the adaptor is zero-allocation and statically dispatched

Presence rules:
- each requested toggle type must appear in the expected leading prefix of the query item
- unsupported or out-of-order toggle requests should fail to compile via trait resolution
- v1 does not rely on runtime type maps or runtime lookup tables for this

--------------------------------------------------
10. INTERNAL ACCESS TRAITS
--------------------------------------------------

The runtime needs an internal helper trait to locate toggleable components in query items.

Conceptually:

    trait HasToggle<T: ToggleableComponent> {
        fn toggle_ref(&self) -> &T;
    }

Implement for:
- direct component refs/wrappers such as `&T`, `&mut T`, `Ref<T>`, and `Mut<T>`
- tuples whose leading components contain the requested toggle types in order

Example supported items:
- `&A`
- `&mut A`
- `(&A, &B)`
- `(&mut A, &B)`
- `(&A, &mut B, &Transform)`
- etc.

Tuple support:
- implement helper traits for tuples up to a fixed arity
- recommended v1 arity: 8
- optional later extension: 16

--------------------------------------------------
11. HOT-PATH FILTER IMPLEMENTATION
--------------------------------------------------

The hot path must avoid:
- heap allocation
- runtime type maps
- boxed closures
- reflection lookup
- hash maps
- trait object dispatch

Implementation strategy:
- fixed tuple arity helper traits
- static generic dispatch
- direct bool comparisons

Example internal shape:

    item_a.is_active() == true &&
    item_b.is_active() == false &&
    item_c.is_active() == true

This is a small branch chain over already-fetched query data, not a structural ECS filter.

--------------------------------------------------
12. CHANGE DETECTION
--------------------------------------------------

Toggling `active` mutates the component normally.
Therefore:
- Bevy change detection on that component will fire as normal
- No special toggle change event is required in v1

Optional future feature:
- a dedicated toggle event stream
- observer integration
- hooks for on-enabled/on-disabled per component type

Not part of v1.

--------------------------------------------------
13. ERROR HANDLING
--------------------------------------------------

Compile-time errors:
- derive on unsupported item
- missing `active`
- `active` not bool
- duplicate requested toggle types in the filter generic list
- requested toggle type not present in the query item when the adaptor is instantiated

Runtime behavior:
- command helpers are best-effort and no-op if the component is missing
- query helpers should not rely on runtime panics for normal misuse

Recommended v1 behavior:
- prefer compile-time failures for filter misuse
- prefer explicit no-op semantics for best-effort command helpers

--------------------------------------------------
14. FEATURES
--------------------------------------------------

Suggested Cargo features:

default = ["derive", "toggle_ext"]

Features:
- derive
  Enables proc-macro derive crate

- toggle_ext
  Enables iterator helpers and command helpers

- debug_checks
  Extra debug assertions and clearer diagnostics for internal invariants

- reflect
  Reserved for future; off in v1

The `ToggleQuery` wrapper SystemParam, if it is ever added, should be treated as a later ergonomics layer rather than part of the locked v1 API.

--------------------------------------------------
15. PRELUDE
--------------------------------------------------

Suggested prelude exports:

- ToggleableComponent
- ToggleEntityCommandsExt
- ToggleIterExt
- `Toggleable` derive macro

--------------------------------------------------
16. EXAMPLE USAGE
--------------------------------------------------

Component:

    #[derive(Component, Toggleable)]
    pub struct Frozen {
        pub active: bool,
        pub stacks: u8,
    }

Direct hot-path mutation:

    fn disable_frozen(mut q: Query<&mut Frozen>) {
        for mut frozen in &mut q {
            frozen.set_active(false);
        }
    }

Commands convenience:

    fn disable_one(mut commands: Commands, q: Query<Entity, With<Frozen>>) {
        for e in &q {
            commands.entity(e).set_toggle_if_present::<Frozen>(false);
        }
    }

Vanilla filtering:

    fn tick(q: Query<(&Frozen, &Burning, &Transform)>) {
        for (frozen, burning, transform) in &q {
            if !(frozen.is_active() && !burning.is_active()) {
                continue;
            }

            // ...
        }
    }

Iterator helper:

    fn tick_filtered(q: Query<(&Frozen, &Burning, &Transform)>) {
        for (frozen, burning, transform) in q
            .iter()
            .where_toggles::<(Frozen, Burning)>((true, false))
        {
            // ...
        }
    }

Single-type convenience:

    fn tick_frozen(q: Query<&Frozen>) {
    for frozen in q.iter().skip_inactive::<Frozen>() {
        // ...
    }

    for frozen in q.iter().skip_active::<Frozen>() {
        // ...
    }
    }

--------------------------------------------------
17. IMPLEMENTATION ORDER
--------------------------------------------------

Phase 1
- runtime trait
- derive macro
- command helpers
- tests for derive errors and toggle mutation behavior

Phase 2
- iterator extension traits
- single-type `skip_inactive::<T>()`
- single-type `skip_active::<T>()`
- multi-type `where_toggles::<(A, ...)>((...))`
- tuple impls up to arity 4 or 8

Phase 3
- debug assertions and diagnostics
- docs/examples/benchmarks

Phase 4
- expand tuple arity if needed
- optional observer/event integrations
- optional reflect support if it proves useful
- optional `ToggleQuery` wrapper only if the ergonomics justify the extra API surface

--------------------------------------------------
18. FINAL POSITIONING
--------------------------------------------------

bevy_toggleables is a small utility crate for per-component active/inactive state.

It exists for users who want:
- very fast logical enable/disable
- zero archetype churn for that state change
- preserved concurrency across many distinct action/modifier component types
- ergonomic filtered iteration when desired

It is intentionally not a replacement for:
- Bevy entity disabling
- structural component presence/absence
- packed bitset state containers
- generic dynamic state registries

It is a focused crate for the specific case:
"keep the component, keep the archetype, just make it logically active or inactive."

use core::iter::FusedIterator;

use bevy_ecs::prelude::{Mut, Ref};

use crate::access::HasToggle;
use crate::ToggleableComponent;

pub trait ToggleTypeList {
    type States;
}

pub trait ToggleItemMatch<Types: ToggleTypeList> {
    fn matches_toggle_states(&self, states: &Types::States) -> bool;
}

#[doc(hidden)]
pub struct ToggleFilter<I, Types: ToggleTypeList> {
    iter: I,
    states: Types::States,
}

impl<I, Types> ToggleFilter<I, Types>
where
    Types: ToggleTypeList,
{
    #[inline]
    pub(crate) fn new(iter: I, states: Types::States) -> Self {
        Self { iter, states }
    }
}

impl<I, Types> Iterator for ToggleFilter<I, Types>
where
    I: Iterator,
    Types: ToggleTypeList,
    I::Item: ToggleItemMatch<Types>,
{
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .by_ref()
            .find(|item| item.matches_toggle_states(&self.states))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.iter.size_hint();
        (0, upper)
    }
}

impl<I, Types> FusedIterator for ToggleFilter<I, Types>
where
    I: FusedIterator,
    Types: ToggleTypeList,
    I::Item: ToggleItemMatch<Types>,
{
}

pub trait ToggleIterExt: Iterator + Sized {
    #[inline]
    fn where_toggles<Types>(
        self,
        states: <Types as ToggleTypeList>::States,
    ) -> ToggleFilter<Self, Types>
    where
        Types: ToggleTypeList,
        Self::Item: ToggleItemMatch<Types>,
    {
        ToggleFilter::new(self, states)
    }

    #[inline]
    fn skip_inactive<T>(self) -> ToggleFilter<Self, (T,)>
    where
        T: ToggleableComponent,
        Self::Item: ToggleItemMatch<(T,)>,
    {
        ToggleFilter::new(self, (true,))
    }

    #[inline]
    fn skip_active<T>(self) -> ToggleFilter<Self, (T,)>
    where
        T: ToggleableComponent,
        Self::Item: ToggleItemMatch<(T,)>,
    {
        ToggleFilter::new(self, (false,))
    }
}

impl<I: Iterator> ToggleIterExt for I {}

impl<A: ToggleableComponent> ToggleTypeList for (A,) {
    type States = (bool,);
}

impl<A: ToggleableComponent, B: ToggleableComponent> ToggleTypeList for (A, B) {
    type States = (bool, bool);
}

impl<A: ToggleableComponent, B: ToggleableComponent, C: ToggleableComponent> ToggleTypeList
    for (A, B, C)
{
    type States = (bool, bool, bool);
}

impl<
        A: ToggleableComponent,
        B: ToggleableComponent,
        C: ToggleableComponent,
        D: ToggleableComponent,
    > ToggleTypeList for (A, B, C, D)
{
    type States = (bool, bool, bool, bool);
}

impl<
        A: ToggleableComponent,
        B: ToggleableComponent,
        C: ToggleableComponent,
        D: ToggleableComponent,
        E: ToggleableComponent,
    > ToggleTypeList for (A, B, C, D, E)
{
    type States = (bool, bool, bool, bool, bool);
}

impl<
        A: ToggleableComponent,
        B: ToggleableComponent,
        C: ToggleableComponent,
        D: ToggleableComponent,
        E: ToggleableComponent,
        F: ToggleableComponent,
    > ToggleTypeList for (A, B, C, D, E, F)
{
    type States = (bool, bool, bool, bool, bool, bool);
}

impl<
        A: ToggleableComponent,
        B: ToggleableComponent,
        C: ToggleableComponent,
        D: ToggleableComponent,
        E: ToggleableComponent,
        F: ToggleableComponent,
        G: ToggleableComponent,
    > ToggleTypeList for (A, B, C, D, E, F, G)
{
    type States = (bool, bool, bool, bool, bool, bool, bool);
}

impl<
        A: ToggleableComponent,
        B: ToggleableComponent,
        C: ToggleableComponent,
        D: ToggleableComponent,
        E: ToggleableComponent,
        F: ToggleableComponent,
        G: ToggleableComponent,
        H: ToggleableComponent,
    > ToggleTypeList for (A, B, C, D, E, F, G, H)
{
    type States = (bool, bool, bool, bool, bool, bool, bool, bool);
}

impl<T> ToggleItemMatch<(T,)> for &T
where
    T: ToggleableComponent,
{
    #[inline]
    fn matches_toggle_states(&self, states: &<(T,) as ToggleTypeList>::States) -> bool {
        self.toggle_ref().is_active() == states.0
    }
}

impl<T> ToggleItemMatch<(T,)> for &mut T
where
    T: ToggleableComponent,
{
    #[inline]
    fn matches_toggle_states(&self, states: &<(T,) as ToggleTypeList>::States) -> bool {
        self.toggle_ref().is_active() == states.0
    }
}

impl<'w, T> ToggleItemMatch<(T,)> for Ref<'w, T>
where
    T: ToggleableComponent,
{
    #[inline]
    fn matches_toggle_states(&self, states: &<(T,) as ToggleTypeList>::States) -> bool {
        self.toggle_ref().is_active() == states.0
    }
}

impl<'w, T> ToggleItemMatch<(T,)> for Mut<'w, T>
where
    T: ToggleableComponent,
{
    #[inline]
    fn matches_toggle_states(&self, states: &<(T,) as ToggleTypeList>::States) -> bool {
        self.toggle_ref().is_active() == states.0
    }
}

include!(concat!(env!("OUT_DIR"), "/toggleable_iter_impls.rs"));

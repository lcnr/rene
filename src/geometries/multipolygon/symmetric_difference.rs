use rithm::big_int::BigInt;
use rithm::fraction::Fraction;

use crate::bounded::{Bounded, Box};
use crate::clipping::{Event, Operation, ReduceEvents, SYMMETRIC_DIFFERENCE};
use crate::geometries::{Empty, Point, Polygon};
use crate::operations::{
    are_boxes_uncoupled, flags_to_false_indices, flags_to_true_indices, merge_boxes,
    to_are_boxes_coupled_with_box,
};
use crate::relatable::Relatable;
use crate::traits::{Elemental, SymmetricDifference};

use super::types::Multipolygon;

impl<Scalar> SymmetricDifference<Empty> for Multipolygon<Scalar> {
    type Output = Self;

    fn symmetric_difference(self, _other: Empty) -> Self::Output {
        self
    }
}

impl<Scalar> SymmetricDifference<&Empty> for Multipolygon<Scalar> {
    type Output = Self;

    fn symmetric_difference(self, _other: &Empty) -> Self::Output {
        self
    }
}

impl<Scalar> SymmetricDifference<Empty> for &Multipolygon<Scalar>
where
    Multipolygon<Scalar>: Clone,
{
    type Output = Multipolygon<Scalar>;

    fn symmetric_difference(self, _other: Empty) -> Self::Output {
        self.clone()
    }
}

impl<Scalar> SymmetricDifference<&Empty> for &Multipolygon<Scalar>
where
    Multipolygon<Scalar>: Clone,
{
    type Output = Multipolygon<Scalar>;

    fn symmetric_difference(self, _other: &Empty) -> Self::Output {
        self.clone()
    }
}

impl<Digit, const SEPARATOR: char, const SHIFT: usize> SymmetricDifference
    for &Multipolygon<Fraction<BigInt<Digit, SEPARATOR, SHIFT>>>
where
    for<'a> &'a Box<Fraction<BigInt<Digit, SEPARATOR, SHIFT>>>: Relatable,
    Fraction<BigInt<Digit, SEPARATOR, SHIFT>>: Clone + Ord,
    Multipolygon<Fraction<BigInt<Digit, SEPARATOR, SHIFT>>>: Bounded<Fraction<BigInt<Digit, SEPARATOR, SHIFT>>>
        + Clone
        + ReduceEvents<
            Point<Fraction<BigInt<Digit, SEPARATOR, SHIFT>>>,
            SYMMETRIC_DIFFERENCE,
            Output = Vec<Polygon<Fraction<BigInt<Digit, SEPARATOR, SHIFT>>>>,
        >,
    for<'a> Operation<Point<Fraction<BigInt<Digit, SEPARATOR, SHIFT>>>, SYMMETRIC_DIFFERENCE>:
        From<(
                &'a [&'a Polygon<Fraction<BigInt<Digit, SEPARATOR, SHIFT>>>],
                &'a [&'a Polygon<Fraction<BigInt<Digit, SEPARATOR, SHIFT>>>],
            )> + Iterator<Item = Event>,
    Point<Fraction<BigInt<Digit, SEPARATOR, SHIFT>>>:
        Elemental<Coordinate = Fraction<BigInt<Digit, SEPARATOR, SHIFT>>>,
    Polygon<Fraction<BigInt<Digit, SEPARATOR, SHIFT>>>:
        Bounded<Fraction<BigInt<Digit, SEPARATOR, SHIFT>>> + Clone,
{
    type Output = Vec<Polygon<Fraction<BigInt<Digit, SEPARATOR, SHIFT>>>>;

    fn symmetric_difference(self, other: Self) -> Self::Output {
        let bounding_boxes = self
            .polygons
            .iter()
            .map(Bounded::to_bounding_box)
            .collect::<Vec<_>>();
        let other_bounding_boxes = other
            .polygons
            .iter()
            .map(Bounded::to_bounding_box)
            .collect::<Vec<_>>();
        let bounding_box = merge_boxes(&bounding_boxes);
        let other_bounding_box = merge_boxes(&other_bounding_boxes);
        if are_boxes_uncoupled(&bounding_box, &other_bounding_box) {
            let mut result = self.polygons.clone();
            result.extend_from_slice(&other.polygons);
            return result;
        }
        let are_bounding_boxes_coupled =
            to_are_boxes_coupled_with_box(&bounding_boxes, &other_bounding_box);
        let coupled_polygons_ids = flags_to_true_indices(&are_bounding_boxes_coupled);
        if coupled_polygons_ids.is_empty() {
            let mut result = self.polygons.clone();
            result.extend_from_slice(&other.polygons);
            return result;
        }
        let are_other_bounding_boxes_coupled =
            to_are_boxes_coupled_with_box(&other_bounding_boxes, &bounding_box);
        let other_coupled_polygons_ids = flags_to_true_indices(&are_other_bounding_boxes_coupled);
        if other_coupled_polygons_ids.is_empty() {
            let mut result = self.polygons.clone();
            result.extend_from_slice(&other.polygons);
            return result;
        }
        let coupled_polygons = coupled_polygons_ids
            .into_iter()
            .map(|index| &self.polygons[index])
            .collect::<Vec<_>>();
        let other_coupled_polygons = other_coupled_polygons_ids
            .into_iter()
            .map(|index| &other.polygons[index])
            .collect::<Vec<_>>();
        let mut operation = Operation::<Point<_>, SYMMETRIC_DIFFERENCE>::from((
            &coupled_polygons,
            &other_coupled_polygons,
        ));
        let mut events = {
            let (_, maybe_events_count) = operation.size_hint();
            debug_assert!(maybe_events_count.is_some());
            Vec::with_capacity(unsafe { maybe_events_count.unwrap_unchecked() })
        };
        while let Some(event) = operation.next() {
            events.push(event)
        }
        let mut result = Multipolygon::<_>::reduce_events(events, &mut operation);
        result.reserve(
            (self.polygons.len() - coupled_polygons.len())
                + (other.polygons.len() - other_coupled_polygons.len()),
        );
        result.extend(
            flags_to_false_indices(&are_bounding_boxes_coupled)
                .into_iter()
                .map(|index| self.polygons[index].clone()),
        );
        result.extend(
            flags_to_false_indices(&are_other_bounding_boxes_coupled)
                .into_iter()
                .map(|index| other.polygons[index].clone()),
        );
        result
    }
}

impl<Digit, const SEPARATOR: char, const SHIFT: usize>
    SymmetricDifference<&Polygon<Fraction<BigInt<Digit, SEPARATOR, SHIFT>>>>
    for &Multipolygon<Fraction<BigInt<Digit, SEPARATOR, SHIFT>>>
where
    for<'a> &'a Box<Fraction<BigInt<Digit, SEPARATOR, SHIFT>>>: Relatable,
    Fraction<BigInt<Digit, SEPARATOR, SHIFT>>: Clone + Ord,
    Multipolygon<Fraction<BigInt<Digit, SEPARATOR, SHIFT>>>: Bounded<Fraction<BigInt<Digit, SEPARATOR, SHIFT>>>
        + ReduceEvents<
            Point<Fraction<BigInt<Digit, SEPARATOR, SHIFT>>>,
            SYMMETRIC_DIFFERENCE,
            Output = Vec<Polygon<Fraction<BigInt<Digit, SEPARATOR, SHIFT>>>>,
        >,
    for<'a> Operation<Point<Fraction<BigInt<Digit, SEPARATOR, SHIFT>>>, SYMMETRIC_DIFFERENCE>:
        From<(
                &'a [&'a Polygon<Fraction<BigInt<Digit, SEPARATOR, SHIFT>>>],
                &'a Polygon<Fraction<BigInt<Digit, SEPARATOR, SHIFT>>>,
            )> + Iterator<Item = Event>,
    Point<Fraction<BigInt<Digit, SEPARATOR, SHIFT>>>:
        Elemental<Coordinate = Fraction<BigInt<Digit, SEPARATOR, SHIFT>>>,
    Polygon<Fraction<BigInt<Digit, SEPARATOR, SHIFT>>>:
        Bounded<Fraction<BigInt<Digit, SEPARATOR, SHIFT>>> + Clone,
{
    type Output = Vec<Polygon<Fraction<BigInt<Digit, SEPARATOR, SHIFT>>>>;

    fn symmetric_difference(
        self,
        other: &Polygon<Fraction<BigInt<Digit, SEPARATOR, SHIFT>>>,
    ) -> Self::Output {
        let bounding_boxes = self
            .polygons
            .iter()
            .map(Bounded::to_bounding_box)
            .collect::<Vec<_>>();
        let bounding_box = merge_boxes(&bounding_boxes);
        let other_bounding_box = other.to_bounding_box();
        if are_boxes_uncoupled(&bounding_box, &other_bounding_box) {
            let mut result = self.polygons.clone();
            result.push(other.clone());
            return result;
        }
        let are_bounding_boxes_coupled =
            to_are_boxes_coupled_with_box(&bounding_boxes, &other_bounding_box);
        let coupled_polygons_ids = flags_to_true_indices(&are_bounding_boxes_coupled);
        if coupled_polygons_ids.is_empty() {
            let mut result = self.polygons.clone();
            result.push(other.clone());
            return result;
        }
        let coupled_polygons = coupled_polygons_ids
            .into_iter()
            .map(|index| &self.polygons[index])
            .collect::<Vec<_>>();
        let mut operation =
            Operation::<Point<_>, SYMMETRIC_DIFFERENCE>::from((&coupled_polygons, other));
        let mut events = {
            let (_, maybe_events_count) = operation.size_hint();
            debug_assert!(maybe_events_count.is_some());
            Vec::with_capacity(unsafe { maybe_events_count.unwrap_unchecked() })
        };
        while let Some(event) = operation.next() {
            events.push(event)
        }
        let mut result = Multipolygon::<_>::reduce_events(events, &mut operation);
        result.reserve(self.polygons.len() - coupled_polygons.len());
        result.extend(
            flags_to_false_indices(&are_bounding_boxes_coupled)
                .into_iter()
                .map(|index| self.polygons[index].clone()),
        );
        result
    }
}

impl<Digit, const SEPARATOR: char, const SHIFT: usize>
    SymmetricDifference<&Multipolygon<Fraction<BigInt<Digit, SEPARATOR, SHIFT>>>>
    for &Polygon<Fraction<BigInt<Digit, SEPARATOR, SHIFT>>>
where
    for<'a> &'a Box<Fraction<BigInt<Digit, SEPARATOR, SHIFT>>>: Relatable,
    for<'a> Operation<Point<Fraction<BigInt<Digit, SEPARATOR, SHIFT>>>, SYMMETRIC_DIFFERENCE>:
        From<(
                &'a Polygon<Fraction<BigInt<Digit, SEPARATOR, SHIFT>>>,
                &'a [&'a Polygon<Fraction<BigInt<Digit, SEPARATOR, SHIFT>>>],
            )> + Iterator<Item = Event>,
    Fraction<BigInt<Digit, SEPARATOR, SHIFT>>: Clone + Ord,
    Multipolygon<Fraction<BigInt<Digit, SEPARATOR, SHIFT>>>:
        Bounded<Fraction<BigInt<Digit, SEPARATOR, SHIFT>>>,
    Point<Fraction<BigInt<Digit, SEPARATOR, SHIFT>>>:
        Elemental<Coordinate = Fraction<BigInt<Digit, SEPARATOR, SHIFT>>>,
    Polygon<Fraction<BigInt<Digit, SEPARATOR, SHIFT>>>: Bounded<Fraction<BigInt<Digit, SEPARATOR, SHIFT>>>
        + Clone
        + ReduceEvents<
            Point<Fraction<BigInt<Digit, SEPARATOR, SHIFT>>>,
            SYMMETRIC_DIFFERENCE,
            Output = Vec<Polygon<Fraction<BigInt<Digit, SEPARATOR, SHIFT>>>>,
        >,
{
    type Output = Vec<Polygon<Fraction<BigInt<Digit, SEPARATOR, SHIFT>>>>;

    fn symmetric_difference(
        self,
        other: &Multipolygon<Fraction<BigInt<Digit, SEPARATOR, SHIFT>>>,
    ) -> Self::Output {
        let other_bounding_boxes = other
            .polygons
            .iter()
            .map(Bounded::to_bounding_box)
            .collect::<Vec<_>>();
        let bounding_box = self.to_bounding_box();
        let other_bounding_box = merge_boxes(&other_bounding_boxes);
        if are_boxes_uncoupled(&bounding_box, &other_bounding_box) {
            let mut result = other.polygons.clone();
            result.push(self.clone());
            return result;
        }
        let are_other_bounding_boxes_coupled =
            to_are_boxes_coupled_with_box(&other_bounding_boxes, &other_bounding_box);
        let other_coupled_polygons_ids = flags_to_true_indices(&are_other_bounding_boxes_coupled);
        if other_coupled_polygons_ids.is_empty() {
            let mut result = other.polygons.clone();
            result.push(self.clone());
            return result;
        }
        let other_coupled_polygons = other_coupled_polygons_ids
            .into_iter()
            .map(|index| &other.polygons[index])
            .collect::<Vec<_>>();
        let mut operation =
            Operation::<Point<_>, SYMMETRIC_DIFFERENCE>::from((self, &other_coupled_polygons));
        let mut events = {
            let (_, maybe_events_count) = operation.size_hint();
            debug_assert!(maybe_events_count.is_some());
            Vec::with_capacity(unsafe { maybe_events_count.unwrap_unchecked() })
        };
        while let Some(event) = operation.next() {
            events.push(event)
        }
        let mut result = Polygon::<_>::reduce_events(events, &mut operation);
        result.reserve(other.polygons.len() - other_coupled_polygons.len());
        result.extend(
            flags_to_false_indices(&are_other_bounding_boxes_coupled)
                .into_iter()
                .map(|index| other.polygons[index].clone()),
        );
        result
    }
}
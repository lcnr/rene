use std::hash::Hash;
use std::ops::{Div, Neg};

use traiter::numbers::Signed;

use crate::bounded;
use crate::bounded::Bounded;
use crate::operations::{
    to_boxes_ids_with_intersection, CrossMultiply, DotMultiply,
    IntersectCrossingSegments, Orient, Square, SquaredMetric,
};
use crate::relatable::{Relatable, Relation};
use crate::sweeping::traits::{EventsQueue, SweepLine};
use crate::traits::{
    Elemental, Iterable, Lengthsome, Multisegmental, Segmental,
};

use super::event::{is_left_event, Event};
use super::linear::Operation;
use super::multisegmental::relate_to_multisegmental;

pub(crate) fn relate_to_contour<
    Contour,
    Multisegment,
    Point: Clone + Hash + Ord,
    Output: Div<Output = Output>
        + Neg<Output = Output>
        + Ord
        + Square<Output = Output>,
    Scalar: Div<Output = Scalar> + Hash + Ord,
    Segment,
>(
    multisegment: &Multisegment,
    contour: &Contour,
) -> Relation
where
    for<'a> &'a Output: Signed,
    for<'a> &'a Contour:
        Bounded<&'a Scalar> + Multisegmental<IndexSegment = Segment>,
    for<'a> &'a Multisegment:
        Bounded<&'a Scalar> + Multisegmental<IndexSegment = Segment>,
    for<'a, 'b> &'a bounded::Box<&'b Scalar>: Relatable,
    for<'a, 'b> &'a Segment:
        Bounded<&'a Scalar> + Segmental<Endpoint = &'a Point>,
    for<'a, 'b> Operation<Point>: From<(&'a [&'b Segment], &'a [&'b Segment])>
        + EventsQueue<Event = Event>
        + SweepLine<Event = Event>,
    for<'a> &'a Point: CrossMultiply<Output = Scalar>
        + DotMultiply<Output = Output>
        + Elemental<Coordinate = &'a Scalar>
        + IntersectCrossingSegments<Output = Point>
        + Orient
        + SquaredMetric<Output = Output>,
{
    relate_to_multisegmental::<
        false,
        true,
        Multisegment,
        Contour,
        Point,
        Output,
        Scalar,
        Segment,
    >(multisegment, contour)
}

pub(crate) fn relate_to_multisegment<
    Multisegment,
    Point: Clone + Hash + Ord,
    Output: Div<Output = Output>
        + Neg<Output = Output>
        + Ord
        + Square<Output = Output>,
    Scalar: Div<Output = Scalar> + Hash + Ord,
    Segment,
>(
    first: &Multisegment,
    second: &Multisegment,
) -> Relation
where
    for<'a> &'a Output: Signed,
    for<'a> &'a Multisegment:
        Bounded<&'a Scalar> + Multisegmental<IndexSegment = Segment>,
    for<'a, 'b> &'a bounded::Box<&'b Scalar>: Relatable,
    for<'a, 'b> &'a Segment:
        Bounded<&'a Scalar> + Segmental<Endpoint = &'a Point>,
    for<'a, 'b> Operation<Point>: From<(&'a [&'b Segment], &'a [&'b Segment])>
        + EventsQueue<Event = Event>
        + SweepLine<Event = Event>,
    for<'a> &'a Point: CrossMultiply<Output = Scalar>
        + DotMultiply<Output = Output>
        + Elemental<Coordinate = &'a Scalar>
        + IntersectCrossingSegments<Output = Point>
        + Orient
        + SquaredMetric<Output = Output>,
{
    relate_to_multisegmental::<
        false,
        false,
        Multisegment,
        Multisegment,
        Point,
        Output,
        Scalar,
        Segment,
    >(first, second)
}

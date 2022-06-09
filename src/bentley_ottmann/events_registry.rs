use core::convert::From;
use std::cmp::Reverse;
use std::collections::{BTreeSet, BinaryHeap};
use std::marker::PhantomData;
use std::ops::Bound::{Excluded, Unbounded};

use rithm::traits::{AdditiveGroup, DivisivePartialMagma, MultiplicativeMonoid, Signed};

use crate::operations::{intersect_crossing_segments, orient, to_sorted_pair};
use crate::oriented::Orientation;
use crate::traits::{Point, Segment};

use super::event::{is_left_event, Event};
use super::events_queue_key::EventsQueueKey;
use super::sweep_line_key::SweepLineKey;
use super::traits::{EventsQueue, SweepLine};

pub(super) struct EventsRegistry<Scalar, Endpoint, const UNIQUE: bool> {
    endpoints: Vec<Endpoint>,
    events_queue_data: BinaryHeap<Reverse<EventsQueueKey<Endpoint>>>,
    min_collinear_segments_ids: Vec<usize>,
    opposites: Vec<Event>,
    segments_ids: Vec<usize>,
    sweep_line_data: BTreeSet<SweepLineKey<Scalar, Endpoint>>,
    _phantom: PhantomData<fn() -> Scalar>,
}

impl<
        Scalar: AdditiveGroup + Clone + DivisivePartialMagma + MultiplicativeMonoid + Ord + Signed,
        Endpoint: Clone + From<(Scalar, Scalar)> + Ord + self::Point<Scalar>,
        const UNIQUE: bool,
    > Iterator for EventsRegistry<Scalar, Endpoint, UNIQUE>
{
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(event) = self.pop() {
            if is_left_event(event) {
                if let Some(equal_segment_event) = <Self as SweepLine>::find(self, event) {
                    self.merge_equal_segment_events(equal_segment_event, event);
                    if UNIQUE {
                        self.next()
                    } else {
                        Some(event)
                    }
                } else {
                    self.insert(event);
                    if let Some(below_event) = self.below(event) {
                        self.detect_intersection(below_event, event);
                    }
                    if let Some(above_event) = self.above(event) {
                        self.detect_intersection(event, above_event);
                    }
                    Some(event)
                }
            } else {
                let event_opposite = self.get_opposite(event);
                debug_assert!(is_left_event(event_opposite));
                if let Some(equal_segment_event) = <Self as SweepLine>::find(self, event_opposite) {
                    let (maybe_above_event, maybe_below_event) = (
                        self.above(equal_segment_event),
                        self.below(equal_segment_event),
                    );
                    self.remove(equal_segment_event);
                    if let (Some(above_event), Some(below_event)) =
                        (maybe_above_event, maybe_below_event)
                    {
                        self.detect_intersection(below_event, above_event);
                    }
                    if equal_segment_event != event_opposite {
                        self.merge_equal_segment_events(event_opposite, equal_segment_event);
                    }
                    Some(event)
                } else {
                    if UNIQUE {
                        self.next()
                    } else {
                        Some(event)
                    }
                }
            }
        } else {
            None
        }
    }
}

impl<Scalar, Endpoint, const UNIQUE: bool> EventsRegistry<Scalar, Endpoint, UNIQUE> {
    pub(super) fn are_collinear(&self, segment_id: usize, other_segment_id: usize) -> bool {
        self.to_min_collinear_segment_id(segment_id)
            == self.to_min_collinear_segment_id(other_segment_id)
    }

    pub(super) fn get_event_end(&self, event: Event) -> &Endpoint {
        &self.endpoints[self.get_opposite(event)]
    }

    pub(super) fn get_event_segment_id(&self, event: Event) -> usize {
        self.to_left_event_segment_id(if is_left_event(event) {
            event
        } else {
            self.get_opposite(event)
        })
    }

    pub(super) fn get_event_start(&self, event: Event) -> &Endpoint {
        &self.endpoints[event]
    }

    pub(super) fn get_segment_end(&self, segment_id: usize) -> &Endpoint {
        &self.endpoints[2 * segment_id + 1]
    }

    pub(super) fn get_segment_start(&self, segment_id: usize) -> &Endpoint {
        &self.endpoints[2 * segment_id]
    }

    fn get_opposite(&self, event: Event) -> Event {
        self.opposites[event]
    }

    fn to_left_event_segment_id(&self, event: Event) -> usize {
        debug_assert!(is_left_event(event));
        self.segments_ids[event / 2]
    }

    fn to_min_collinear_segment_id(&self, segment_id: usize) -> usize {
        let mut candidate = segment_id;
        let mut iterations_count = 0;
        while self.min_collinear_segments_ids[candidate] != candidate {
            candidate = self.min_collinear_segments_ids[candidate];
            iterations_count += 1;
        }
        debug_assert!(iterations_count < 3);
        candidate
    }

    fn to_sweep_line_key(&self, event: Event) -> SweepLineKey<Scalar, Endpoint> {
        debug_assert!(is_left_event(event));
        SweepLineKey::new(event, &self.endpoints, &self.opposites)
    }
}

impl<
        Scalar,
        Endpoint: Ord,
        Segment: self::Segment<Scalar, Point = Endpoint>,
        const UNIQUE: bool,
    > From<&[Segment]> for EventsRegistry<Scalar, Endpoint, UNIQUE>
{
    fn from(segments: &[Segment]) -> Self {
        let capacity = 2 * segments.len();
        let mut result = Self {
            endpoints: Vec::with_capacity(capacity),
            events_queue_data: BinaryHeap::with_capacity(capacity),
            min_collinear_segments_ids: (0..segments.len()).collect(),
            opposites: Vec::with_capacity(capacity),
            segments_ids: (0..segments.len()).collect(),
            sweep_line_data: BTreeSet::new(),
            _phantom: PhantomData,
        };
        for (index, segment) in segments.iter().enumerate() {
            let (start, end) = to_sorted_pair((segment.start(), segment.end()));
            let left_event = 2 * index;
            let right_event = 2 * index + 1;
            result.endpoints.push(start);
            result.endpoints.push(end);
            result.opposites.push(right_event);
            result.opposites.push(left_event);
            result.push(left_event);
            result.push(right_event);
        }
        result
    }
}

impl<
        Scalar: AdditiveGroup + Clone + DivisivePartialMagma + MultiplicativeMonoid + Ord + Signed,
        Endpoint: Clone + From<(Scalar, Scalar)> + Ord + self::Point<Scalar>,
        const UNIQUE: bool,
    > EventsRegistry<Scalar, Endpoint, UNIQUE>
{
    pub(super) fn detect_intersection(&mut self, below_event: Event, event: Event) {
        debug_assert_ne!(below_event, event);

        let event_start = self.get_event_start(event);
        let event_end = self.get_event_end(event);
        let below_event_start = self.get_event_start(below_event);
        let below_event_end = self.get_event_end(below_event);

        let event_start_orientation = orient(below_event_end, below_event_start, event_start);
        let event_end_orientation = orient(below_event_end, below_event_start, event_end);
        if event_start_orientation != Orientation::Collinear
            && event_end_orientation != Orientation::Collinear
        {
            if event_start_orientation != event_end_orientation {
                let below_event_start_orientation =
                    orient(event_start, event_end, below_event_start);
                let below_event_end_orientation = orient(event_start, event_end, below_event_end);
                if below_event_start_orientation != Orientation::Collinear
                    && below_event_end_orientation != Orientation::Collinear
                {
                    if below_event_start_orientation != below_event_end_orientation {
                        let point = intersect_crossing_segments(
                            event_start,
                            event_end,
                            below_event_start,
                            below_event_end,
                        );
                        self.divide_event_by_midpoint(below_event, point.clone());
                        self.divide_event_by_midpoint_checking_above(event, point);
                    }
                } else if below_event_start_orientation != Orientation::Collinear {
                    if event_start < below_event_end && below_event_end < event_end {
                        let point = below_event_end.clone();
                        self.divide_event_by_midpoint_checking_above(event, point);
                    }
                } else if event_start < below_event_start && below_event_start < event_end {
                    let point = below_event_start.clone();
                    self.divide_event_by_midpoint_checking_above(event, point);
                }
            }
        } else if event_end_orientation != Orientation::Collinear {
            if below_event_start < event_start && event_start < below_event_end {
                let point = event_start.clone();
                self.divide_event_by_midpoint(below_event, point);
            }
        } else if event_start_orientation != Orientation::Collinear {
            if below_event_start < event_end && event_end < below_event_end {
                let point = event_end.clone();
                self.divide_event_by_midpoint(below_event, point);
            }
        } else if event_start == below_event_start {
            debug_assert!(event_end != below_event_end);
            let (max_end_event, min_end_event) = if event_end < below_event_end {
                (below_event, event)
            } else {
                (event, below_event)
            };
            self.remove(max_end_event);
            let min_end = self.get_event_end(min_end_event).clone();
            let (_, min_end_max_end_event) = self.divide(max_end_event, min_end);
            self.push(min_end_max_end_event);
            self.merge_equal_segment_events(event, below_event);
        } else if event_end == below_event_end {
            let (max_start_event, min_start_event) = if event_start < below_event_start {
                (below_event, event)
            } else {
                (event, below_event)
            };
            let max_start = self.get_event_start(max_start_event).clone();
            let (max_start_to_min_start_event, max_start_to_end_event) =
                self.divide(min_start_event, max_start);
            self.push(max_start_to_min_start_event);
            self.merge_equal_segment_events(max_start_event, max_start_to_end_event);
        } else if below_event_start < event_start && event_start < below_event_end {
            if event_end < below_event_end {
                let event_start = event_start.clone();
                let event_end = event_end.clone();
                self.divide_event_by_mid_segment_event_endpoints(
                    below_event,
                    event,
                    event_start,
                    event_end,
                );
            } else {
                let (max_start, min_end) = (event_start.clone(), below_event_end.clone());
                self.divide_overlapping_events(below_event, event, max_start, min_end);
            }
        } else if event_start < below_event_start && below_event_start < event_end {
            if below_event_end < event_end {
                let below_event_start = below_event_start.clone();
                let below_event_end = below_event_end.clone();
                self.divide_event_by_mid_segment_event_endpoints(
                    event,
                    below_event,
                    below_event_start,
                    below_event_end,
                );
            } else {
                let max_start = below_event_start.clone();
                let min_end = event_end.clone();
                self.divide_overlapping_events(event, below_event, max_start, min_end);
            }
        }
    }

    pub(super) fn merge_equal_segment_events(&mut self, first: Event, second: Event) {
        debug_assert_ne!(first, second);
        debug_assert!(is_left_event(first));
        debug_assert!(is_left_event(second));
        debug_assert_ne!(
            self.to_left_event_segment_id(first),
            self.to_left_event_segment_id(second)
        );
        debug_assert!(self.get_event_start(first) == self.get_event_start(second));
        debug_assert!(self.get_event_end(first) == self.get_event_end(second));

        let first_segment_id = self.to_left_event_segment_id(first);
        let second_segment_id = self.to_left_event_segment_id(second);
        let first_min_collinear_segment_id = self.min_collinear_segments_ids[first_segment_id];
        let second_min_collinear_segment_id = self.min_collinear_segments_ids[second_segment_id];
        let min_collinear_segment_id =
            first_min_collinear_segment_id.min(second_min_collinear_segment_id);
        self.min_collinear_segments_ids[first_segment_id] = min_collinear_segment_id;
        self.min_collinear_segments_ids[second_segment_id] = min_collinear_segment_id;
        self.min_collinear_segments_ids[first_min_collinear_segment_id] = min_collinear_segment_id;
        self.min_collinear_segments_ids[second_min_collinear_segment_id] = min_collinear_segment_id;
    }

    fn divide_overlapping_events(
        &mut self,
        min_start_event: Event,
        max_start_event: Event,
        max_start: Endpoint,
        min_end: Endpoint,
    ) {
        self.divide_event_by_midpoint(max_start_event, min_end);
        let (max_start_min_start_event, max_start_min_end_event) =
            self.divide(min_start_event, max_start);
        self.push(max_start_min_start_event);
        self.merge_equal_segment_events(max_start_event, max_start_min_end_event);
    }

    fn divide_event_by_mid_segment_event_endpoints(
        &mut self,
        event: Event,
        mid_segment_event: Event,
        mid_segment_event_start: Endpoint,
        mid_segment_event_end: Endpoint,
    ) where
        Endpoint: PartialEq,
    {
        debug_assert!(mid_segment_event_start.eq(self.get_event_start(mid_segment_event)));
        debug_assert!(mid_segment_event_end.eq(self.get_event_end(mid_segment_event)));
        debug_assert!(mid_segment_event_start.ne(self.get_event_start(event)));
        debug_assert!(mid_segment_event_end.ne(self.get_event_end(event)));

        self.divide_event_by_midpoint(event, mid_segment_event_end);
        let (
            inner_subsegment_event_start_to_composite_event_start_index,
            inner_subsegment_event_start_to_inner_subsegment_event_end_index,
        ) = self.divide(event, mid_segment_event_start);
        self.push(inner_subsegment_event_start_to_composite_event_start_index);
        self.merge_equal_segment_events(
            mid_segment_event,
            inner_subsegment_event_start_to_inner_subsegment_event_end_index,
        );
    }

    fn divide_event_by_midpoint(&mut self, event: Event, point: Endpoint) {
        let (point_to_event_start_index, point_to_event_end_index) = self.divide(event, point);
        self.push(point_to_event_start_index);
        self.push(point_to_event_end_index);
    }

    fn divide_event_by_midpoint_checking_above(&mut self, event: Event, point: Endpoint) {
        if let Some(above_event) = self.above(event) {
            if self
                .get_event_start(above_event)
                .eq(self.get_event_start(event))
                && self.get_event_end(above_event).eq(&point)
            {
                self.remove(above_event);
                self.divide_event_by_midpoint(event, point);
                self.merge_equal_segment_events(event, above_event);
                return;
            }
        }
        self.divide_event_by_midpoint(event, point);
    }
}

impl<Scalar, Endpoint: Clone + self::Point<Scalar> + Ord, const UNIQUE: bool>
    EventsRegistry<Scalar, Endpoint, UNIQUE>
{
    pub(super) fn divide(&mut self, event: Event, mid_point: Endpoint) -> (Event, Event) {
        debug_assert!(is_left_event(event));
        let opposite_event = self.get_opposite(event);
        let mid_point_to_event_end_event = self.endpoints.len();
        self.segments_ids.push(self.to_left_event_segment_id(event));
        self.endpoints.push(mid_point.clone());
        self.opposites.push(opposite_event);
        self.opposites[opposite_event] = mid_point_to_event_end_event;
        let mid_point_to_event_start_event = self.endpoints.len();
        self.endpoints.push(mid_point);
        self.opposites.push(event);
        self.opposites[event] = mid_point_to_event_start_event;
        (mid_point_to_event_start_event, mid_point_to_event_end_event)
    }
}

impl<Scalar, Endpoint: Ord, const UNIQUE: bool> EventsQueue
    for EventsRegistry<Scalar, Endpoint, UNIQUE>
{
    fn pop(&mut self) -> Option<Event> {
        self.events_queue_data.pop().map(|key| key.0.event)
    }

    fn push(&mut self, event: Event) {
        self.events_queue_data.push(Reverse(EventsQueueKey::new(
            event,
            &self.endpoints,
            &self.opposites,
        )))
    }
}

impl<
        Scalar: AdditiveGroup + MultiplicativeMonoid + Ord + Signed,
        Endpoint: Clone + Eq + Point<Scalar>,
        const UNIQUE: bool,
    > SweepLine for EventsRegistry<Scalar, Endpoint, UNIQUE>
{
    fn above(&self, event: Event) -> Option<Event> {
        self.sweep_line_data
            .range((Excluded(&self.to_sweep_line_key(event)), Unbounded))
            .next()
            .map(|key| key.event)
    }

    fn below(&self, event: Event) -> Option<Event> {
        self.sweep_line_data
            .range((Unbounded, Excluded(&self.to_sweep_line_key(event))))
            .last()
            .map(|key| key.event)
    }

    fn find(&self, event: Event) -> Option<Event> {
        self.sweep_line_data
            .get(&self.to_sweep_line_key(event))
            .map(|key| key.event)
    }

    fn insert(&mut self, event: Event) -> bool {
        self.sweep_line_data.insert(self.to_sweep_line_key(event))
    }

    fn remove(&mut self, event: Event) -> bool {
        self.sweep_line_data.remove(&self.to_sweep_line_key(event))
    }
}

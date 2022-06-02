use core::convert::From;

use rithm::traits::{AdditiveGroup, DivisivePartialMagma, MultiplicativeMonoid, Signed};

use crate::traits::{Point, Segment};

use super::event::is_left_event;
use super::events_queue::EventsQueue;
use super::sweep_line::SweepLine;

pub(crate) fn sweep<
    Scalar: AdditiveGroup + Clone + DivisivePartialMagma + MultiplicativeMonoid + Ord + Signed,
    Endpoint: Clone + From<(Scalar, Scalar)> + Ord + self::Point<Scalar>,
    Segment: From<(Endpoint, Endpoint)> + self::Segment<Scalar, Point = Endpoint>,
>(
    segments: &[Segment],
) -> Vec<Segment> {
    let mut result = Vec::with_capacity(segments.len());
    let mut endpoints = Vec::with_capacity(2 * segments.len());
    let mut opposites = Vec::with_capacity(2 * segments.len());
    let mut events_queue = EventsQueue::new(&mut endpoints, &mut opposites, segments);
    let mut sweep_line = SweepLine::new(&endpoints, &opposites);
    while let Some(event) = events_queue.pop() {
        if is_left_event(event) {
            if let None = sweep_line.find(event) {
                sweep_line.insert(event);
                if let Some(below_event) = sweep_line.below(event) {
                    events_queue.detect_intersection(below_event, event, &mut sweep_line);
                }
                if let Some(above_event) = sweep_line.above(event) {
                    events_queue.detect_intersection(event, above_event, &mut sweep_line);
                }
            }
        } else {
            let event = events_queue.get_opposite(event);
            if let Some(equal_segment_event) = sweep_line.find(event) {
                let (maybe_above_event, maybe_below_event) = (
                    sweep_line.above(equal_segment_event),
                    sweep_line.below(equal_segment_event),
                );
                sweep_line.remove(equal_segment_event);
                match (maybe_above_event, maybe_below_event) {
                    (Some(above_event), Some(below_event)) => {
                        events_queue.detect_intersection(below_event, above_event, &mut sweep_line);
                    }
                    _ => {}
                }
                result.push(Segment::from((
                    events_queue.get_event_start(equal_segment_event).clone(),
                    events_queue.get_event_end(equal_segment_event).clone(),
                )));
            }
        }
    }
    result
}
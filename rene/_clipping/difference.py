from typing import List

from rene._utils import (do_boxes_have_no_common_area,
                         flags_to_false_indices,
                         flags_to_true_indices,
                         merge_boxes,
                         to_boxes_have_common_area,
                         to_boxes_ids_with_common_area)
from rene.hints import (Multipolygon,
                        Polygon)
from .event import Event
from .operation import Operation


class Difference(Operation):
    def _detect_if_left_event_from_result(self, event: Event) -> bool:
        return (self._is_outside_left_event(event)
                if self._is_left_event_from_first_operand(event)
                else (self._is_inside_left_event(event)
                      or self._is_common_polyline_component_left_event(event)))


def subtract_multipolygons(first: Multipolygon,
                           second: Multipolygon) -> List[Polygon]:
    first_polygons, second_polygons = first.polygons, second.polygons
    first_boxes = [polygon.bounding_box for polygon in first_polygons]
    second_boxes = [polygon.bounding_box for polygon in second_polygons]
    first_bounding_box, second_bounding_box = (merge_boxes(first_boxes),
                                               merge_boxes(second_boxes))
    if do_boxes_have_no_common_area(first_bounding_box, second_bounding_box):
        return list(first.polygons)
    first_boxes_have_common_area = to_boxes_have_common_area(
            first_boxes, second_bounding_box
    )
    first_common_area_polygons_ids = flags_to_true_indices(
            first_boxes_have_common_area
    )
    if not first_common_area_polygons_ids:
        return list(first.polygons)
    second_common_area_polygons_ids = to_boxes_ids_with_common_area(
            second_boxes, first_bounding_box
    )
    if not second_common_area_polygons_ids:
        return list(first.polygons)
    first_common_area_polygons = [
        first_polygons[polygon_id]
        for polygon_id in first_common_area_polygons_ids
    ]
    second_common_area_polygons = [
        second_polygons[polygon_id]
        for polygon_id in second_common_area_polygons_ids
    ]
    max_x = max(first_boxes[polygon_id].max_x
                for polygon_id in first_common_area_polygons_ids)
    operation = Difference.from_multisegmentals_sequences(
            first_common_area_polygons, second_common_area_polygons
    )
    events = []
    for event in operation:
        if operation.to_event_start(event).x > max_x:
            break
        events.append(event)
    result = operation.reduce_events(events, type(first_polygons[0].border),
                                     type(first_polygons[0]))
    result.extend(
            first_polygons[index]
            for index in flags_to_false_indices(
                    first_boxes_have_common_area
            )
    )
    return result


def subtract_polygon_from_multipolygon(first: Multipolygon,
                                       second: Polygon) -> List[Polygon]:
    first_polygons = first.polygons
    first_boxes = [polygon.bounding_box for polygon in first_polygons]
    first_bounding_box, second_bounding_box = (
        merge_boxes(first_boxes), second.bounding_box
    )
    if do_boxes_have_no_common_area(first_bounding_box, second_bounding_box):
        return list(first.polygons)
    first_boxes_have_common_area = to_boxes_have_common_area(
            first_boxes, second_bounding_box
    )
    first_common_area_polygons_ids = flags_to_true_indices(
            first_boxes_have_common_area
    )
    if not first_common_area_polygons_ids:
        return list(first.polygons)
    first_common_area_polygons = [
        first_polygons[polygon_id]
        for polygon_id in first_common_area_polygons_ids
    ]
    max_x = max(first_boxes[polygon_id].max_x
                for polygon_id in first_common_area_polygons_ids)
    operation = Difference.from_multisegmentals_sequence_multisegmental(
            first_common_area_polygons, second
    )
    events = []
    for event in operation:
        if operation.to_event_start(event).x > max_x:
            break
        events.append(event)
    result = operation.reduce_events(events, type(first_polygons[0].border),
                                     type(first_polygons[0]))
    result.extend(
            first_polygons[index]
            for index in flags_to_false_indices(
                    first_boxes_have_common_area
            )
    )
    return result


def subtract_multipolygon_from_polygon(first: Polygon,
                                       second: Multipolygon) -> List[Polygon]:
    second_polygons = second.polygons
    second_boxes = [polygon.bounding_box for polygon in second_polygons]
    first_bounding_box, second_bounding_box = (first.bounding_box,
                                               merge_boxes(second_boxes))
    if do_boxes_have_no_common_area(first_bounding_box, second_bounding_box):
        return [first]
    second_common_area_polygons_ids = to_boxes_ids_with_common_area(
            second_boxes, first_bounding_box
    )
    if not second_common_area_polygons_ids:
        return [first]
    second_common_area_polygons = [
        second_polygons[polygon_id]
        for polygon_id in second_common_area_polygons_ids
    ]
    max_x = first_bounding_box.max_x
    operation = Difference.from_multisegmental_multisegmentals_sequence(
            first, second_common_area_polygons
    )
    events = []
    for event in operation:
        if operation.to_event_start(event).x > max_x:
            break
        events.append(event)
    return operation.reduce_events(events, type(first.border), type(first))


def subtract_polygons(first: Polygon, second: Polygon) -> List[Polygon]:
    first_bounding_box, second_bounding_box = (first.bounding_box,
                                               second.bounding_box)
    if do_boxes_have_no_common_area(first_bounding_box, second_bounding_box):
        return [first]
    operation = Difference.from_multisegmentals(first, second)
    max_x = first_bounding_box.max_x
    events = []
    for event in operation:
        if operation.to_event_start(event).x > max_x:
            break
        events.append(event)
    return operation.reduce_events(events, type(first.border), type(first))

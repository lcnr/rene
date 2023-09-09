import typing as t

import typing_extensions as te

from rene import (Orientation,
                  hints)
from rene._hints import Map
from rene._utils import orient
from .event import (Event,
                    is_event_left)


class EventsQueueKey(t.Generic[hints.Scalar]):
    event: Event
    is_from_first_operand: bool
    endpoints: Map[Event, hints.Point[hints.Scalar]]
    opposites: Map[Event, Event]

    __slots__ = 'endpoints', 'event', 'is_from_first_operand', 'opposites'

    def __new__(cls,
                event: Event,
                is_from_first_operand: bool,
                endpoints: Map[Event, hints.Point[hints.Scalar]],
                opposites: Map[Event, Event],
                /) -> te.Self:
        self = super().__new__(cls)
        (
            self.endpoints, self.event, self.is_from_first_operand,
            self.opposites
        ) = endpoints, event, is_from_first_operand, opposites
        return self

    def __lt__(self, other: te.Self, /) -> bool:
        """
        Checks if the event should be processed before the other.
        """
        event, other_event = self.event, other.event
        event_start, other_event_start = (self.endpoints[event],
                                          self.endpoints[other_event])
        start_x, start_y = event_start.x, event_start.y
        other_start_x, other_start_y = other_event_start.x, other_event_start.y
        if start_x != other_start_x:
            # different x-coordinate,
            # the event with lower x-coordinate is processed first
            return start_x < other_start_x
        elif start_y != other_start_y:
            # different starts, but same x-coordinate,
            # the event with lower y-coordinate is processed first
            return start_y < other_start_y
        elif is_event_left(event) is not is_event_left(other_event):
            # same start, but one is a left endpoint
            # and the other is a right endpoint,
            # the right endpoint is processed first
            return not is_event_left(event)
        else:
            other_end_orientation = orient(
                    event_start, self.endpoints[self.opposites[event]],
                    self.endpoints[self.opposites[other_event]],
            )
            if other_end_orientation is Orientation.COLLINEAR:
                assert (self.is_from_first_operand
                        is not other.is_from_first_operand)
                return other.is_from_first_operand
            else:
                return (other_end_orientation
                        # the lowest segment is processed first
                        is (Orientation.COUNTERCLOCKWISE
                            if is_event_left(event)
                            else Orientation.CLOCKWISE))

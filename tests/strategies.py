from functools import partial
from operator import (attrgetter,
                      itemgetter)

from ground.base import (Context,
                         Mode)
from hypothesis import strategies
from hypothesis_geometry import planar
from rithm import (Fraction,
                   Int)

from rene.exact import (Contour,
                        Point,
                        Polygon,
                        Segment)

context = Context(contour_cls=Contour,
                  point_cls=Point,
                  polygon_cls=Polygon,
                  segment_cls=Segment,
                  mode=Mode.PLAIN)
MAX_VALUE = 10 ** 10
MIN_VALUE = -MAX_VALUE
integers = strategies.builds(Int, strategies.integers(MIN_VALUE, MAX_VALUE))
non_zero_integers = strategies.builds(Int,
                                      strategies.integers(MIN_VALUE, -1)
                                      | strategies.integers(1, MAX_VALUE))
scalars = (integers | strategies.builds(Fraction, integers, non_zero_integers)
           | strategies.floats(MIN_VALUE, MAX_VALUE,
                               allow_infinity=False,
                               allow_nan=False))
to_contours = partial(planar.contours, scalars, scalars,
                      context=context)
to_polygons = partial(planar.polygons, scalars, scalars,
                      context=context)
segments = planar.segments(scalars,
                           context=context)
points = planar.points(scalars,
                       context=context)
contours = to_contours()
contours_vertices = contours.map(attrgetter('vertices')).map(itemgetter(0))
polygons = to_polygons()
polygons_components = polygons.map(attrgetter('border', 'holes'))

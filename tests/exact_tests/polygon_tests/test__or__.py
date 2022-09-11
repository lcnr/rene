from hypothesis import given

from rene.exact import (Multipolygon,
                        Polygon)
from tests.utils import (Compound,
                         reverse_polygon_holes)
from . import strategies


@given(strategies.polygons, strategies.compounds)
def test_basic(first: Polygon, second: Compound) -> None:
    result = first | second

    assert isinstance(result, (Multipolygon, Polygon))


@given(strategies.polygons, strategies.compounds)
def test_reversals(first: Polygon, second: Compound) -> None:
    result = first | second

    assert result == reverse_polygon_holes(first) | second

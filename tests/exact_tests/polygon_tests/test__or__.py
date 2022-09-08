from hypothesis import given

from rene.exact import (Multipolygon,
                        Polygon)
from tests.utils import (reverse_compound_coordinates,
                         reverse_polygon_coordinates,
                         reverse_polygon_holes)
from . import strategies


@given(strategies.polygons, strategies.polygons)
def test_basic(first: Polygon, second: Polygon) -> None:
    result = first | second

    assert isinstance(result, (Multipolygon, Polygon))


@given(strategies.polygons)
def test_idempotence(polygon: Polygon) -> None:
    assert polygon | polygon == polygon


@given(strategies.polygons, strategies.polygons)
def test_absorption_identity(first: Polygon, second: Polygon) -> None:
    assert first | (first & second) == first


@given(strategies.polygons, strategies.polygons)
def test_commutativity(first: Polygon, second: Polygon) -> None:
    assert first | second == second | first


@given(strategies.polygons, strategies.polygons, strategies.polygons)
def test_associativity(first: Polygon,
                       second: Polygon,
                       third: Polygon) -> None:
    assert (first | second) | third == first | second | third


@given(strategies.polygons, strategies.polygons, strategies.polygons)
def test_difference_operand(first: Polygon,
                            second: Polygon,
                            third: Polygon) -> None:
    assert (first - second) | third == (first | third) - (second - third)


@given(strategies.polygons, strategies.polygons, strategies.polygons)
def test_distribution_over_intersection(first: Polygon,
                                        second: Polygon,
                                        third: Polygon) -> None:
    assert first | (second & third) == (first | second) & (first | third)


@given(strategies.polygons, strategies.polygons)
def test_equivalents(first: Polygon, second: Polygon) -> None:
    assert first | second == (first ^ second) ^ (first & second)


@given(strategies.polygons, strategies.polygons)
def test_reversals(first: Polygon, second: Polygon) -> None:
    result = first | second

    assert result == reverse_polygon_holes(first) | second
    assert result == first | reverse_polygon_holes(second)
    assert result == reverse_compound_coordinates(
            reverse_polygon_coordinates(first)
            | reverse_polygon_coordinates(second)
    )
from hypothesis import given

from rene.exact import (Empty,
                        Multipolygon,
                        Polygon)
from tests.utils import (Compound,
                         reverse_multipolygon)
from . import strategies


@given(strategies.multipolygons, strategies.compounds)
def test_basic(first: Multipolygon, second: Compound) -> None:
    result = first ^ second

    assert isinstance(result, (Empty, Multipolygon, Polygon))


@given(strategies.multipolygons, strategies.compounds)
def test_reversals(first: Multipolygon, second: Compound) -> None:
    result = first ^ second

    assert result == reverse_multipolygon(first) ^ second
from numbers import Rational as _Rational
from typing import (Any as _Any,
                    Sequence as _Sequence,
                    TypeVar as _TypeVar,
                    overload as _overload)

from typing_extensions import Protocol as _Protocol

from rene import Orientation as _Orientation

Scalar = _TypeVar('Scalar',
                  bound=_Rational)


class Point(_Protocol[Scalar]):
    @property
    def x(self) -> Scalar:
        ...

    @property
    def y(self) -> Scalar:
        ...

    def __new__(cls, x: Scalar, y: Scalar) -> 'Point[Scalar]':
        ...

    @_overload
    def __eq__(self, other: 'Point[Scalar]') -> bool:
        ...

    @_overload
    def __eq__(self, other: _Any) -> _Any:
        ...

    def __eq__(self, other):
        ...

    def __ge__(self, other: 'Point[Scalar]') -> bool:
        ...

    def __gt__(self, other: 'Point[Scalar]') -> bool:
        ...

    def __hash__(self) -> int:
        ...

    def __le__(self, other: 'Point[Scalar]') -> bool:
        ...

    def __lt__(self, other: 'Point[Scalar]') -> bool:
        ...

    def __repr__(self) -> str:
        ...

    def __str__(self) -> str:
        ...


class Segment(_Protocol[Scalar]):
    @property
    def end(self) -> Point[Scalar]:
        ...

    @property
    def start(self) -> Point[Scalar]:
        ...

    def __new__(cls,
                start: Point[Scalar],
                end: Point[Scalar]) -> 'Segment[Scalar]':
        ...

    @_overload
    def __eq__(self, other: 'Segment[Scalar]') -> bool:
        ...

    @_overload
    def __eq__(self, other: _Any) -> _Any:
        ...

    def __hash__(self) -> int:
        ...

    def __repr__(self) -> str:
        ...

    def __str__(self) -> str:
        ...


class Contour(_Protocol[Scalar]):
    @property
    def orientation(self) -> _Orientation:
        ...

    @property
    def segments(self) -> _Sequence[Segment[Scalar]]:
        ...

    @property
    def vertices(self) -> _Sequence[Point[Scalar]]:
        ...

    def is_valid(self) -> bool:
        ...

    def __new__(cls, vertices: _Sequence[Point[Scalar]]) -> 'Contour[Scalar]':
        ...

    @_overload
    def __eq__(self, other: 'Contour[Scalar]') -> bool:
        ...

    @_overload
    def __eq__(self, other: _Any) -> _Any:
        ...

    def __hash__(self) -> int:
        ...

    def __repr__(self) -> str:
        ...

    def __str__(self) -> str:
        ...


class Polygon(_Protocol[Scalar]):
    @property
    def border(self) -> Contour[Scalar]:
        ...

    @property
    def holes(self) -> _Sequence[Contour[Scalar]]:
        ...

    def __new__(cls,
                border: Contour[Scalar],
                holes: _Sequence[Contour[Scalar]]) -> 'Polygon[Scalar]':
        ...

    @_overload
    def __eq__(self, other: 'Polygon[Scalar]') -> bool:
        ...

    @_overload
    def __eq__(self, other: _Any) -> _Any:
        ...

    def __repr__(self) -> str:
        ...

    def __str__(self) -> str:
        ...
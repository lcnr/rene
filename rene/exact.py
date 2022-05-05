try:
    from ._exact import Point
except ImportError:
    from rithm import Fraction as _Fraction


    class Point:
        @property
        def x(self):
            return self._x

        @property
        def y(self):
            return self._y

        __slots__ = '_x', '_y'

        def __new__(cls, x, y):
            self = super().__new__(cls)
            self._x, self._y = (_Fraction(x)
                                if isinstance(x, float)
                                else _Fraction(x.numerator, x.denominator),
                                _Fraction(y)
                                if isinstance(y, float)
                                else _Fraction(y.numerator, y.denominator))
            return self

        def __eq__(self, other):
            return (self.x == other.x and self.y == other.y
                    if isinstance(other, Point)
                    else NotImplemented)

        def __repr__(self):
            return (f'{__name__}.{type(self).__qualname__}'
                    f'({self.x!r}, {self.y!r})')

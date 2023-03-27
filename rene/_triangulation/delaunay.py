from __future__ import annotations

import typing as _t

import typing_extensions as _te
from reprit.base import generate_repr

from rene._rene import Orientation
from rene._utils import deduplicate
from rene.hints import (Point,
                        Scalar)
from .mesh import (Mesh,
                   build_delaunay_triangulation,
                   orient_point_to_edge)
from .quad_edge import (QuadEdge,
                        to_opposite_edge)


class DelaunayTriangulation(_t.Generic[Scalar]):
    @classmethod
    def from_points(cls, points: _t.Sequence[Point[Scalar]]) -> _te.Self:
        endpoints = list(points)
        endpoints.sort()
        mesh = Mesh.from_points(deduplicate(endpoints))
        left_side, right_side = build_delaunay_triangulation(mesh)
        return cls(left_side, right_side, mesh)

    @property
    def left_side(self) -> QuadEdge:
        return self._left_side

    @property
    def mesh(self) -> Mesh[Scalar]:
        return self._mesh

    @property
    def right_side(self) -> QuadEdge:
        return self._right_side

    def to_boundary_points(self) -> _t.List[Point[Scalar]]:
        if self:
            result = []
            start = self.left_side
            edge = start
            while True:
                result.append(self.mesh.to_start(edge))
                candidate = self.mesh.to_right_from_end(edge)
                if candidate == start:
                    break
                edge = candidate
            return result
        else:
            return self.mesh.endpoints

    def triangles_vertices(
            self
    ) -> _t.List[_t.Tuple[Point[Scalar], Point[Scalar], Point[Scalar]]]:
        mesh = self.mesh
        result = []
        for edge in mesh.to_edges():
            first_vertex = mesh.to_start(edge)
            second_vertex = mesh.to_end(edge)
            third_vertex = mesh.to_end(mesh.to_left_from_start(edge))
            if (first_vertex < second_vertex
                    and first_vertex < third_vertex
                    and third_vertex == mesh.to_end(
                            mesh.to_right_from_start(to_opposite_edge(edge))
                    )
                    and orient_point_to_edge(
                            mesh, edge, third_vertex
                    ) is Orientation.COUNTERCLOCKWISE):
                result.append((first_vertex, second_vertex, third_vertex))
        return result

    __slots__ = '_left_side', '_mesh', '_right_side'

    def __init__(self,
                 left_side: QuadEdge,
                 right_side: QuadEdge,
                 mesh: Mesh[Scalar]) -> None:
        self._left_side, self._mesh, self._right_side = (
            left_side, mesh, right_side
        )

    __repr__ = generate_repr(__init__)

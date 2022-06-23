use std::marker::PhantomData;

use crate::delaunay::contracts::UNDEFINED_INDEX;
use crate::delaunay::quad_edge::{to_opposite_edge, to_rotated_edge, QuadEdge};

pub(super) struct QuadEdgeRegistry<Scalar, Endpoint> {
    endpoints: Vec<Endpoint>,
    left_from_start: Vec<QuadEdge>,
    starts_indices: Vec<usize>,
    _phantom: PhantomData<fn() -> Scalar>,
}

impl<Scalar, Endpoint> From<Vec<Endpoint>> for QuadEdgeRegistry<Scalar, Endpoint> {
    fn from(endpoints: Vec<Endpoint>) -> Self {
        let endpoints_count = endpoints.len();
        Self {
            endpoints,
            left_from_start: Vec::with_capacity(4 * endpoints_count),
            starts_indices: Vec::with_capacity(4 * endpoints_count),
            _phantom: PhantomData,
        }
    }
}

impl<Scalar, Endpoint> QuadEdgeRegistry<Scalar, Endpoint> {
    pub(super) fn get_start(&self, edge: QuadEdge) -> &Endpoint {
        &self.endpoints[self.to_start_index(edge)]
    }

    pub(super) fn get_end(&self, edge: QuadEdge) -> &Endpoint {
        &self.endpoints[self.to_start_index(to_opposite_edge(edge))]
    }

    pub(super) fn to_left_from_start(&self, edge: QuadEdge) -> QuadEdge {
        self.left_from_start[edge]
    }

    pub(super) fn to_left_from_end(&self, edge: QuadEdge) -> QuadEdge {
        to_rotated_edge(self.to_left_from_start(to_opposite_edge(to_rotated_edge(edge))))
    }

    pub(super) fn to_right_from_end(&self, edge: QuadEdge) -> QuadEdge {
        self.to_left_from_start(to_opposite_edge(edge))
    }

    pub(super) fn to_right_from_start(&self, edge: QuadEdge) -> QuadEdge {
        to_rotated_edge(self.to_left_from_start(to_rotated_edge(edge)))
    }

    fn to_end_index(&self, edge: QuadEdge) -> usize {
        self.starts_indices[to_opposite_edge(edge)]
    }

    fn to_start_index(&self, edge: QuadEdge) -> usize {
        let result = self.starts_indices[edge];
        debug_assert_ne!(result, UNDEFINED_INDEX);
        result
    }
}

impl<Scalar, Endpoint> QuadEdgeRegistry<Scalar, Endpoint> {
    pub(super) fn connect_edges(&mut self, first: QuadEdge, second: QuadEdge) -> QuadEdge {
        let result = self.create_edge(self.to_end_index(first), self.to_start_index(second));
        self.splice_edges(result, self.to_left_from_end(first));
        self.splice_edges(to_opposite_edge(result), second);
        result
    }

    pub(super) fn create_edge(&mut self, start_index: usize, end_index: usize) -> QuadEdge {
        self.starts_indices.push(start_index);
        self.starts_indices.push(UNDEFINED_INDEX);
        self.starts_indices.push(end_index);
        self.starts_indices.push(UNDEFINED_INDEX);
        let edge = self.left_from_start.len();
        let rotated_edge = edge + 1;
        let opposite_edge = edge + 2;
        let triple_rotated_edge = edge + 3;
        self.left_from_start.push(edge);
        self.left_from_start.push(triple_rotated_edge);
        self.left_from_start.push(opposite_edge);
        self.left_from_start.push(rotated_edge);
        edge
    }

    pub(super) fn delete_edge(&mut self, edge: QuadEdge) {
        self.splice_edges(edge, self.to_right_from_start(edge));
        let opposite_edge = to_opposite_edge(edge);
        self.splice_edges(opposite_edge, self.to_right_from_start(opposite_edge));
    }

    pub(super) fn splice_edges(&mut self, first: QuadEdge, second: QuadEdge) {
        let alpha = to_rotated_edge(self.to_left_from_start(first));
        let beta = to_rotated_edge(self.to_left_from_start(second));
        (self.left_from_start[first], self.left_from_start[second]) = (
            self.to_left_from_start(second),
            self.to_left_from_start(first),
        );
        (self.left_from_start[alpha], self.left_from_start[beta]) = (
            self.to_left_from_start(beta),
            self.to_left_from_start(alpha),
        );
    }
}

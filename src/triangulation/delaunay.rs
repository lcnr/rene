use crate::constants::MIN_CONTOUR_VERTICES_COUNT;
use crate::operations::{shrink_collinear_vertices, Orient};

use super::mesh::Mesh;
use super::operations::{BoundaryEndpoints, DelaunayTriangulatable};
use super::quad_edge::{QuadEdge, UNDEFINED_QUAD_EDGE};

#[derive(Clone)]
pub(crate) struct DelaunayTriangulation<Endpoint> {
    left_side: QuadEdge,
    mesh: Mesh<Endpoint>,
    right_side: QuadEdge,
}

impl<Endpoint: Clone + Orient> BoundaryEndpoints<Endpoint> for DelaunayTriangulation<Endpoint> {
    fn to_boundary_points(&self) -> Vec<Endpoint> {
        let endpoints = self.mesh.get_endpoints();
        if endpoints.len() < MIN_CONTOUR_VERTICES_COUNT {
            endpoints.to_vec()
        } else {
            let mut result = Vec::new();
            let start = self.left_side;
            let mut edge = start;
            loop {
                result.push(self.mesh.get_start(edge));
                let candidate = self.mesh.to_right_from_end(edge);
                if candidate == start {
                    break;
                }
                edge = candidate;
            }
            shrink_collinear_vertices(&result)
                .into_iter()
                .cloned()
                .collect()
        }
    }
}

impl<Endpoint: Ord> From<Vec<Endpoint>> for DelaunayTriangulation<Endpoint>
where
    Mesh<Endpoint>: DelaunayTriangulatable,
{
    fn from(mut endpoints: Vec<Endpoint>) -> Self {
        endpoints.sort();
        endpoints.dedup();
        let mut mesh = Mesh::from(endpoints);
        let (left_side, right_side) = mesh.delaunay_triangulation();
        Self {
            left_side,
            mesh,
            right_side,
        }
    }
}

impl<Endpoint> DelaunayTriangulation<Endpoint> {
    pub(crate) fn is_empty(&self) -> bool {
        let result = self.mesh.is_empty();
        debug_assert_eq!(self.left_side == UNDEFINED_QUAD_EDGE, result);
        debug_assert_eq!(self.right_side == UNDEFINED_QUAD_EDGE, result);
        result
    }
}

impl<Endpoint: Clone + Orient + PartialOrd> DelaunayTriangulation<Endpoint> {
    pub(crate) fn to_triangles_vertices(&self) -> Vec<[Endpoint; 3]> {
        self.mesh.to_triangles_vertices()
    }
}

use crate::geometries::Segment;
use crate::slice_sequence::SliceSequence;
use crate::traits::Multisegmental2;

use super::types::Contour;

impl<'a, Scalar> Multisegmental2 for &'a Contour<Scalar> {
    type IndexSegment = Segment<Scalar>;
    type IntoIteratorSegment = &'a Segment<Scalar>;
    type Segments = SliceSequence<'a, Segment<Scalar>>;

    fn segments2(self) -> Self::Segments {
        SliceSequence::new(&self.segments)
    }
}

impl<Scalar> Multisegmental2 for Contour<Scalar> {
    type IndexSegment = Segment<Scalar>;
    type IntoIteratorSegment = Segment<Scalar>;
    type Segments = Vec<Segment<Scalar>>;

    fn segments2(self) -> Self::Segments {
        self.segments
    }
}

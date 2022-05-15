use std::cmp::Ordering;
use std::collections::hash_map::RandomState;
use std::collections::hash_set::HashSet;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;

use rithm::traits::{AdditiveGroup, MultiplicativeMonoid, Sign, Signed};

use crate::oriented::{Orientation, Oriented};

use super::traits;

#[derive(Clone, fmt::Debug)]
pub struct Point<Scalar>(Scalar, Scalar);

impl<Scalar> Point<Scalar> {
    pub fn new(x: Scalar, y: Scalar) -> Self {
        Self(x, y)
    }
}

impl<Scalar: fmt::Display> fmt::Display for Point<Scalar> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_fmt(format_args!("Point({}, {})", self.0, self.1))
    }
}

impl<Scalar: Eq> Eq for Point<Scalar> {}

impl<Scalar: Hash> Hash for Point<Scalar> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        self.1.hash(state);
    }
}

impl<Scalar: Ord> Ord for Point<Scalar> {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.lt(other) {
            Ordering::Less
        } else if self.gt(other) {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

impl<Scalar: PartialEq> PartialEq for Point<Scalar> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }

    fn ne(&self, other: &Self) -> bool {
        self.0 != other.0 || self.1 != other.1
    }
}

impl<Scalar: PartialOrd> PartialOrd for Point<Scalar> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(if self.lt(other) {
            Ordering::Less
        } else if self.gt(other) {
            Ordering::Greater
        } else {
            Ordering::Equal
        })
    }

    fn ge(&self, other: &Self) -> bool {
        self.0.ge(&other.0) || self.0.eq(&other.0) && self.1.ge(&other.1)
    }

    fn gt(&self, other: &Self) -> bool {
        self.0.gt(&other.0) || self.0.eq(&other.0) && self.1.gt(&other.1)
    }

    fn le(&self, other: &Self) -> bool {
        self.0.lt(&other.0) || self.0.eq(&other.0) && self.1.le(&other.1)
    }

    fn lt(&self, other: &Self) -> bool {
        self.0.lt(&other.0) || self.0.eq(&other.0) && self.1.lt(&other.1)
    }
}

impl<Scalar: Clone> traits::Point<Scalar> for Point<Scalar> {
    fn x(&self) -> Scalar {
        self.0.clone()
    }

    fn y(&self) -> Scalar {
        self.1.clone()
    }
}

#[derive(Clone)]
pub struct Segment<Scalar>(Point<Scalar>, Point<Scalar>);

impl<Scalar> Segment<Scalar> {
    pub fn new(start: Point<Scalar>, end: Point<Scalar>) -> Self {
        Self(start, end)
    }
}

impl<Scalar: PartialOrd + Hash> Hash for Segment<Scalar> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if self.0.lt(&self.1) {
            self.0.hash(state);
            self.1.hash(state);
        } else {
            self.1.hash(state);
            self.0.hash(state);
        }
    }
}

impl<Scalar: Clone> traits::Segment<Scalar> for Segment<Scalar> {
    type Point = self::Point<Scalar>;

    fn start(&self) -> Self::Point {
        self.0.clone()
    }

    fn end(&self) -> Self::Point {
        self.1.clone()
    }
}

impl<Scalar: PartialEq> PartialEq for Segment<Scalar> {
    fn eq(&self, other: &Self) -> bool {
        (self.0 == other.0 && self.1 == other.1) || (self.1 == other.0 && self.0 == other.1)
    }

    fn ne(&self, other: &Self) -> bool {
        (self.0 != other.0 && self.1 != other.0) || (self.0 != other.1 && self.1 != other.1)
    }
}

impl<Scalar: Eq> Eq for Segment<Scalar> {}

#[derive(Clone)]
pub struct Contour<Scalar>(Vec<Point<Scalar>>);

impl<Scalar: Clone> Contour<Scalar> {
    pub fn new(vertices: Vec<Point<Scalar>>) -> Self {
        Self(vertices)
    }
}

impl<Scalar: Eq> Eq for Contour<Scalar> {}

impl<Scalar: AdditiveGroup + Clone + Hash + MultiplicativeMonoid + Ord + Signed> Hash
    for Contour<Scalar>
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        let min_vertex_index = self.to_min_vertex_index();
        self.0[min_vertex_index].hash(state);
        match self.orientation() {
            Orientation::Clockwise => {
                for index in (0..min_vertex_index).rev() {
                    self.0[index].hash(state);
                }
                for index in (min_vertex_index + 1..self.0.len()).rev() {
                    self.0[index].hash(state);
                }
            }
            _ => {
                for index in min_vertex_index + 1..self.0.len() {
                    self.0[index].hash(state);
                }
                for index in 0..min_vertex_index {
                    self.0[index].hash(state);
                }
            }
        }
    }
}

fn cross_multiply<Scalar: AdditiveGroup + MultiplicativeMonoid>(
    first_start: Point<Scalar>,
    first_end: Point<Scalar>,
    second_start: Point<Scalar>,
    second_end: Point<Scalar>,
) -> Scalar {
    (first_end.0 - first_start.0) * (second_end.1 - second_start.1)
        - (first_end.1 - first_start.1) * (second_end.0 - second_start.0)
}

impl<Scalar: AdditiveGroup + Clone + MultiplicativeMonoid + Ord + Signed> Oriented
    for Contour<Scalar>
{
    fn orientation(&self) -> Orientation {
        let min_vertex_index = self.to_min_vertex_index();
        let previous_to_min_vertex_index = if min_vertex_index.is_zero() {
            self.0.len() - 1
        } else {
            min_vertex_index - 1
        };
        let next_to_min_vertex_index = unsafe {
            (min_vertex_index + 1)
                .checked_rem_euclid(self.0.len())
                .unwrap_unchecked()
        };
        sign_to_orientation(
            cross_multiply(
                self.0[previous_to_min_vertex_index].clone(),
                self.0[min_vertex_index].clone(),
                self.0[previous_to_min_vertex_index].clone(),
                self.0[next_to_min_vertex_index].clone(),
            )
            .sign(),
        )
    }
}

impl<Scalar: PartialEq> PartialEq for Contour<Scalar> {
    fn eq(&self, other: &Self) -> bool {
        are_non_empty_unique_sequences_rotationally_equivalent(&self.0, &other.0)
    }

    fn ne(&self, other: &Self) -> bool {
        !are_non_empty_unique_sequences_rotationally_equivalent(&self.0, &other.0)
    }
}

fn are_non_empty_unique_sequences_rotationally_equivalent<T: PartialEq>(
    left: &[T],
    right: &[T],
) -> bool {
    debug_assert!(!left.is_empty() && !right.is_empty());
    if left.len() != right.len() {
        false
    } else {
        let left_first_element = &left[0];
        right
            .iter()
            .position(|value| value == left_first_element)
            .map(|index| {
                (left[1..left.len() - index] == right[index + 1..]
                    && left[left.len() - index..] == right[..index])
                    || (left[left.len() - index..]
                        .iter()
                        .rev()
                        .eq(right[..index].iter())
                        && left[1..left.len() - index]
                            .iter()
                            .rev()
                            .eq(right[index + 1..].iter()))
            })
            .unwrap_or(false)
    }
}

impl<Scalar: Clone> traits::Contour<Scalar> for Contour<Scalar> {
    type Point = self::Point<Scalar>;
    type Segment = self::Segment<Scalar>;

    fn vertices(&self) -> Vec<Self::Point> {
        self.0.clone()
    }

    fn segments(&self) -> Vec<Self::Segment> {
        let mut result = Vec::<Self::Segment>::with_capacity(self.0.len());
        for index in 0..self.0.len() - 1 {
            result.push(Segment(self.0[index].clone(), self.0[index + 1].clone()))
        }
        result.push(Segment(self.0[self.0.len() - 1].clone(), self.0[0].clone()));
        result
    }
}

#[derive(Clone)]
pub struct Polygon<Scalar>(Contour<Scalar>, Vec<Contour<Scalar>>);

impl<Scalar: AdditiveGroup + Clone + Eq + Hash + MultiplicativeMonoid + Ord + Signed> PartialEq
    for Polygon<Scalar>
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
            && are_unique_hashable_sequences_permutationally_equivalent(&self.1, &other.1)
    }

    fn ne(&self, other: &Self) -> bool {
        self.0 != other.0
            || !are_unique_hashable_sequences_permutationally_equivalent(&self.1, &other.1)
    }
}

fn are_unique_hashable_sequences_permutationally_equivalent<T: Eq + Hash>(
    left: &[T],
    right: &[T],
) -> bool {
    if left.len() != right.len() {
        false
    } else {
        let left_set = HashSet::<_, RandomState>::from_iter(left);
        right.iter().all(|value| left_set.contains(value))
    }
}

impl<Scalar: Clone> traits::Polygon<Scalar> for Polygon<Scalar> {
    type Point = self::Point<Scalar>;
    type Segment = self::Segment<Scalar>;
    type Contour = self::Contour<Scalar>;

    fn border(&self) -> Self::Contour {
        self.0.clone()
    }

    fn holes(&self) -> Vec<Self::Contour> {
        self.1.clone()
    }
}

#[derive(Clone)]
struct Multipolygon<Scalar>(Vec<Polygon<Scalar>>);

impl<Scalar: Clone> traits::Multipolygon<Scalar> for Multipolygon<Scalar> {
    type Point = self::Point<Scalar>;
    type Segment = self::Segment<Scalar>;
    type Contour = self::Contour<Scalar>;
    type Polygon = self::Polygon<Scalar>;

    fn polygons(&self) -> Vec<Self::Polygon> {
        self.0.clone()
    }
}

#[inline(always)]
fn sign_to_orientation(sign: Sign) -> Orientation {
    match sign {
        Sign::Negative => Orientation::Clockwise,
        Sign::Positive => Orientation::Counterclockwise,
        Sign::Zero => Orientation::Collinear,
    }
}

impl<Scalar: Ord> Contour<Scalar> {
    fn to_min_vertex_index(&self) -> usize {
        unsafe {
            (0..self.0.len())
                .min_by_key(|index| &self.0[*index])
                .unwrap_unchecked()
        }
    }
}
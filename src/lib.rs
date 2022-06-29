#![feature(build_hasher_simple_hash_one)]
#![feature(map_first_last)]

use std::cmp::Ordering;
use std::convert::TryFrom;

use pyo3::basic::CompareOp;
use pyo3::exceptions::{PyOverflowError, PyTypeError, PyValueError, PyZeroDivisionError};
use pyo3::prelude::{pyclass, pymethods, pymodule, PyModule, PyResult, Python};
use pyo3::type_object::PyTypeObject;
use pyo3::types::{PyFloat, PyFrozenSet, PyLong, PySequence, PyTuple};
use pyo3::{
    ffi, intern, AsPyPointer, FromPyObject, IntoPy, Py, PyAny, PyErr, PyObject, ToPyObject,
};
use rithm::traits::{Endianness, FromBytes, ToBytes, Zeroable};
use rithm::{big_int, fraction};

use crate::bentley_ottmann::{is_contour_valid, to_unique_non_crossing_or_overlapping_segments};
use crate::geometries::MIN_CONTOUR_VERTICES_COUNT;
use crate::operations::to_arg_min;
use crate::oriented::{Orientation, Oriented};
use crate::relatable::Relation;
use crate::traits::{Contour, Point, Polygon, Segment};

mod bentley_ottmann;
mod delaunay;
pub mod geometries;
mod iteration;
pub mod locatable;
mod operations;
pub mod oriented;
pub mod relatable;
pub mod traits;

#[cfg(target_arch = "x86")]
type Digit = u16;
#[cfg(not(target_arch = "x86"))]
type Digit = u32;

const BINARY_SHIFT: usize = (Digit::BITS - 1) as usize;

type BigInt = big_int::BigInt<Digit, '_', BINARY_SHIFT>;
type Fraction = fraction::Fraction<BigInt>;
type ExactContour = geometries::Contour<Fraction>;
type ExactPoint = geometries::Point<Fraction>;
type ExactPolygon = geometries::Polygon<Fraction>;
type ExactSegment = geometries::Segment<Fraction>;

impl IntoPy<PyObject> for ExactContour {
    fn into_py(self, py: Python<'_>) -> PyObject {
        PyExactContour(self).into_py(py)
    }
}

impl ToPyObject for ExactPoint {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        self.clone().into_py(py)
    }
}

impl IntoPy<PyObject> for ExactPoint {
    fn into_py(self, py: Python<'_>) -> PyObject {
        PyExactPoint(self).into_py(py)
    }
}

impl IntoPy<PyObject> for ExactSegment {
    fn into_py(self, py: Python<'_>) -> PyObject {
        PyExactSegment(self).into_py(py)
    }
}

impl From<PyExactContour> for ExactContour {
    fn from(value: PyExactContour) -> Self {
        value.0
    }
}

impl From<PyExactPoint> for ExactPoint {
    fn from(value: PyExactPoint) -> Self {
        value.0
    }
}

impl From<PyExactPolygon> for ExactPolygon {
    fn from(value: PyExactPolygon) -> Self {
        value.0
    }
}

impl From<PyExactSegment> for ExactSegment {
    fn from(value: PyExactSegment) -> Self {
        value.0
    }
}

#[pyclass(name = "Orientation", module = "rene")]
#[derive(Clone)]
struct PyOrientation(Orientation);

#[pymethods]
impl PyOrientation {
    #[classattr]
    const CLOCKWISE: PyOrientation = PyOrientation(Orientation::Clockwise);

    #[classattr]
    const COLLINEAR: PyOrientation = PyOrientation(Orientation::Collinear);

    #[classattr]
    const COUNTERCLOCKWISE: PyOrientation = PyOrientation(Orientation::Counterclockwise);

    fn __repr__(&self) -> String {
        format!(
            "rene.Orientation.{}",
            match self.0 {
                Orientation::Clockwise => "CLOCKWISE",
                Orientation::Collinear => "COLLINEAR",
                Orientation::Counterclockwise => "COUNTERCLOCKWISE",
            }
        )
    }

    fn __str__(&self) -> String {
        format!(
            "Orientation.{}",
            match self.0 {
                Orientation::Clockwise => "CLOCKWISE",
                Orientation::Collinear => "COLLINEAR",
                Orientation::Counterclockwise => "COUNTERCLOCKWISE",
            }
        )
    }
}

#[pyclass(name = "Relation", module = "rene")]
#[derive(Clone)]
struct PyRelation(Relation);

#[pymethods]
impl PyRelation {
    #[classattr]
    const COMPONENT: PyRelation = PyRelation(Relation::Component);

    #[classattr]
    const COMPOSITE: PyRelation = PyRelation(Relation::Composite);

    #[classattr]
    const COVER: PyRelation = PyRelation(Relation::Cover);

    #[classattr]
    const CROSS: PyRelation = PyRelation(Relation::Cross);

    #[classattr]
    const DISJOINT: PyRelation = PyRelation(Relation::Disjoint);

    #[classattr]
    const ENCLOSED: PyRelation = PyRelation(Relation::Enclosed);

    #[classattr]
    const ENCLOSES: PyRelation = PyRelation(Relation::Encloses);

    #[classattr]
    const EQUAL: PyRelation = PyRelation(Relation::Equal);

    #[classattr]
    const OVERLAP: PyRelation = PyRelation(Relation::Overlap);

    #[classattr]
    const TOUCH: PyRelation = PyRelation(Relation::Touch);

    #[classattr]
    const WITHIN: PyRelation = PyRelation(Relation::Within);

    fn __repr__(&self) -> String {
        format!(
            "rene.Relation.{}",
            match self.0 {
                Relation::Component => "COMPONENT",
                Relation::Composite => "COMPOSITE",
                Relation::Cover => "COVER",
                Relation::Cross => "CROSS",
                Relation::Disjoint => "DISJOINT",
                Relation::Enclosed => "ENCLOSED",
                Relation::Encloses => "ENCLOSES",
                Relation::Equal => "EQUAL",
                Relation::Overlap => "OVERLAP",
                Relation::Touch => "TOUCH",
                Relation::Within => "WITHIN",
            }
        )
    }

    fn __str__(&self) -> String {
        format!(
            "Relation.{}",
            match self.0 {
                Relation::Component => "COMPONENT",
                Relation::Composite => "COMPOSITE",
                Relation::Cover => "COVER",
                Relation::Cross => "CROSS",
                Relation::Disjoint => "DISJOINT",
                Relation::Enclosed => "ENCLOSED",
                Relation::Encloses => "ENCLOSES",
                Relation::Equal => "EQUAL",
                Relation::Overlap => "OVERLAP",
                Relation::Touch => "TOUCH",
                Relation::Within => "WITHIN",
            }
        )
    }
}

#[pyclass(name = "Contour", module = "rene.exact", subclass)]
#[pyo3(text_signature = "(vertices, /)")]
#[derive(Clone)]
struct PyExactContour(ExactContour);

#[pyclass(name = "Point", module = "rene.exact", subclass)]
#[pyo3(text_signature = "(x, y, /)")]
#[derive(Clone)]
struct PyExactPoint(ExactPoint);

#[pyclass(name = "Polygon", module = "rene.exact", subclass)]
#[pyo3(text_signature = "(border, holes, /)")]
#[derive(Clone)]
struct PyExactPolygon(ExactPolygon);

#[pyclass(name = "Segment", module = "rene.exact", subclass)]
#[pyo3(text_signature = "(start, end, /)")]
#[derive(Clone)]
struct PyExactSegment(ExactSegment);

#[pymethods]
impl PyExactContour {
    #[new]
    fn new(vertices: &PySequence) -> PyResult<Self> {
        let vertices = extract_from_sequence::<PyExactPoint, ExactPoint>(vertices)?;
        if vertices.len() < MIN_CONTOUR_VERTICES_COUNT {
            Err(PyValueError::new_err(format!(
                "Contour should have at least {} vertices, but found {}.",
                MIN_CONTOUR_VERTICES_COUNT,
                vertices.len()
            )))
        } else {
            Ok(PyExactContour(ExactContour::new(vertices)))
        }
    }

    #[getter]
    fn segments(&self) -> Vec<ExactSegment> {
        self.0.segments()
    }

    #[getter]
    fn vertices(&self) -> Vec<ExactPoint> {
        self.0.vertices()
    }

    #[getter]
    fn orientation(&self, py: Python) -> PyResult<&PyAny> {
        let orientation = self.0.orientation();
        let orientation_cls =
            unsafe { ROOT_MODULE.unwrap_unchecked() }.getattr(intern!(py, "Orientation"))?;
        match orientation {
            Orientation::Clockwise => orientation_cls.getattr(intern!(py, "CLOCKWISE")),
            Orientation::Collinear => orientation_cls.getattr(intern!(py, "COLLINEAR")),
            Orientation::Counterclockwise => {
                orientation_cls.getattr(intern!(py, "COUNTERCLOCKWISE"))
            }
        }
    }

    fn is_valid(&self) -> bool {
        is_contour_valid(&self.0)
    }

    fn __hash__(&self, py: Python) -> PyResult<ffi::Py_hash_t> {
        let mut vertices = self.0.vertices();
        let min_vertex_index = unsafe { to_arg_min(&vertices).unwrap_unchecked() };
        vertices.rotate_left(min_vertex_index);
        if self.0.orientation() == Orientation::Clockwise {
            vertices[1..].reverse()
        }
        PyTuple::new(py, &vertices).hash()
    }

    fn __repr__(&self, py: Python) -> PyResult<String> {
        Ok(format!(
            "rene.exact.Contour({})",
            self.vertices()
                .into_py(py)
                .as_ref(py)
                .repr()?
                .extract::<String>()?
        ))
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        let py = other.py();
        if other.is_instance(PyExactContour::type_object(py))? {
            let other = other.extract::<PyExactContour>()?;
            match op {
                CompareOp::Eq => Ok((self.0 == other.0).into_py(py)),
                CompareOp::Ne => Ok((self.0 != other.0).into_py(py)),
                _ => Ok(py.NotImplemented()),
            }
        } else {
            Ok(py.NotImplemented())
        }
    }

    fn __str__(&self, py: Python) -> PyResult<String> {
        Ok(format!(
            "Contour([{}])",
            self.vertices()
                .into_iter()
                .flat_map(|vertex| PyExactPoint(vertex).__str__(py))
                .collect::<Vec<String>>()
                .join(", ")
        ))
    }
}

#[pymethods]
impl PyExactPoint {
    #[new]
    fn new(x: &PyAny, y: &PyAny) -> PyResult<Self> {
        Ok(PyExactPoint(ExactPoint::new(
            try_scalar_to_fraction(x)?,
            try_scalar_to_fraction(y)?,
        )))
    }

    #[getter]
    fn x<'a>(&self, py: Python<'a>) -> PyResult<&'a PyAny> {
        try_fraction_to_py_fraction(self.0.x(), py)
    }

    #[getter]
    fn y<'a>(&self, py: Python<'a>) -> PyResult<&'a PyAny> {
        try_fraction_to_py_fraction(self.0.y(), py)
    }

    fn __hash__(&self, py: Python) -> PyResult<ffi::Py_hash_t> {
        PyTuple::new(py, [self.x(py)?, self.y(py)?]).hash()
    }

    fn __repr__(&self, py: Python) -> PyResult<String> {
        Ok(format!(
            "rene.exact.Point({}, {})",
            self.x(py)?.repr()?.extract::<String>()?,
            self.y(py)?.repr()?.extract::<String>()?,
        ))
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        let py = other.py();
        if other.is_instance(PyExactPoint::type_object(py))? {
            let other = other.extract::<PyExactPoint>()?;
            match op {
                CompareOp::Eq => Ok((self.0 == other.0).into_py(py)),
                CompareOp::Ge => Ok((self.0 >= other.0).into_py(py)),
                CompareOp::Gt => Ok((self.0 > other.0).into_py(py)),
                CompareOp::Le => Ok((self.0 <= other.0).into_py(py)),
                CompareOp::Lt => Ok((self.0 < other.0).into_py(py)),
                CompareOp::Ne => Ok((self.0 != other.0).into_py(py)),
            }
        } else {
            Ok(py.NotImplemented())
        }
    }

    fn __str__(&self, py: Python) -> PyResult<String> {
        Ok(format!(
            "Point({}, {})",
            self.x(py)?.str()?.extract::<String>()?,
            self.y(py)?.str()?.extract::<String>()?,
        ))
    }
}

#[pymethods]
impl PyExactPolygon {
    #[new]
    fn new(border: &PyExactContour, holes: &PySequence) -> PyResult<Self> {
        Ok(PyExactPolygon(ExactPolygon::new(
            border.0.clone(),
            extract_from_sequence::<PyExactContour, ExactContour>(holes)?,
        )))
    }

    #[getter]
    fn border(&self) -> ExactContour {
        self.0.border()
    }

    #[getter]
    fn holes(&self) -> Vec<ExactContour> {
        self.0.holes()
    }

    fn __repr__(&self, py: Python) -> PyResult<String> {
        Ok(format!(
            "rene.exact.Polygon({}, {})",
            PyExactContour(self.border()).__repr__(py)?,
            self.holes()
                .into_py(py)
                .as_ref(py)
                .repr()?
                .extract::<String>()?
        ))
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        let py = other.py();
        if other.is_instance(PyExactPolygon::type_object(py))? {
            let other = other.extract::<PyExactPolygon>()?;
            match op {
                CompareOp::Eq => Ok((self.0 == other.0).into_py(py)),
                CompareOp::Ne => Ok((self.0 != other.0).into_py(py)),
                _ => Ok(py.NotImplemented()),
            }
        } else {
            Ok(py.NotImplemented())
        }
    }

    fn __str__(&self, py: Python) -> PyResult<String> {
        Ok(format!(
            "Polygon({}, [{}])",
            PyExactContour(self.border()).__str__(py)?,
            self.holes()
                .into_iter()
                .flat_map(|hole| PyExactContour(hole).__str__(py))
                .collect::<Vec<String>>()
                .join(", ")
        ))
    }
}

#[pymethods]
impl PyExactSegment {
    #[new]
    fn new(start: &PyExactPoint, end: &PyExactPoint) -> PyResult<Self> {
        Ok(PyExactSegment(ExactSegment::new(
            start.0.clone(),
            end.0.clone(),
        )))
    }

    #[getter]
    fn start(&self) -> PyExactPoint {
        PyExactPoint(self.0.start())
    }

    #[getter]
    fn end(&self) -> PyExactPoint {
        PyExactPoint(self.0.end())
    }

    fn __hash__(&self, py: Python) -> PyResult<ffi::Py_hash_t> {
        PyFrozenSet::new(py, &[self.start().into_py(py), self.end().into_py(py)])?.hash()
    }

    fn __repr__(&self, py: Python) -> PyResult<String> {
        Ok(format!(
            "rene.exact.Segment({}, {})",
            self.start().__repr__(py)?,
            self.end().__repr__(py)?,
        ))
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        let py = other.py();
        if other.is_instance(PyExactSegment::type_object(py))? {
            let other = other.extract::<PyExactSegment>()?;
            match op {
                CompareOp::Eq => Ok((self.0 == other.0).into_py(py)),
                CompareOp::Ne => Ok((self.0 != other.0).into_py(py)),
                _ => Ok(py.NotImplemented()),
            }
        } else {
            Ok(py.NotImplemented())
        }
    }

    fn __str__(&self, py: Python) -> PyResult<String> {
        Ok(format!(
            "Segment({}, {})",
            self.start().__str__(py)?,
            self.end().__str__(py)?,
        ))
    }
}

fn try_fraction_to_py_fraction(value: Fraction, py: Python) -> PyResult<&PyAny> {
    let rithm_module = py.import("rithm")?;
    let fraction_cls = rithm_module.getattr("Fraction")?;
    fraction_cls.call(
        (
            big_int_to_py_long(value.numerator()),
            big_int_to_py_long(value.denominator()),
        ),
        None,
    )
}

#[inline]
fn big_int_to_py_long(value: &BigInt) -> PyObject {
    let buffer = value.to_bytes(Endianness::Little);
    Python::with_gil(|py| unsafe {
        PyObject::from_owned_ptr(
            py,
            ffi::_PyLong_FromByteArray(buffer.as_ptr(), buffer.len(), 1, 1),
        )
    })
}

#[inline]
fn try_py_integral_to_big_int(value: &PyAny) -> PyResult<BigInt> {
    let ptr = value.as_ptr();
    let py = value.py();
    unsafe {
        let ptr = ffi::PyNumber_Long(ptr);
        if ptr.is_null() {
            return Err(PyErr::fetch(py));
        }
        let bits_count = ffi::_PyLong_NumBits(ptr);
        match bits_count.cmp(&0) {
            Ordering::Less => Err(PyErr::fetch(py)),
            Ordering::Equal => Ok(BigInt::zero()),
            Ordering::Greater => {
                let bytes_count = (bits_count as usize) / (u8::BITS as usize) + 1;
                let mut buffer = vec![0u8; bytes_count];
                if ffi::_PyLong_AsByteArray(
                    Py::<PyLong>::from_owned_ptr(py, ptr).as_ptr() as *mut ffi::PyLongObject,
                    buffer.as_mut_ptr(),
                    buffer.len(),
                    1,
                    1,
                ) < 0
                {
                    Err(PyErr::fetch(py))
                } else {
                    Ok(BigInt::from_bytes(
                        buffer.as_mut_slice(),
                        Endianness::Little,
                    ))
                }
            }
        }
    }
}

const INVALID_SCALAR_TYPE_ERROR_MESSAGE: &str = "Scalar should be a rational number.";
const UNDEFINED_DIVISION_ERROR_MESSAGE: &str = "Division by zero is undefined.";

fn try_scalar_to_fraction(value: &PyAny) -> PyResult<Fraction> {
    let py = value.py();
    if value.is_instance(PyFloat::type_object(py))? {
        Fraction::try_from(value.extract::<f64>()?).map_err(|reason| match reason {
            fraction::FromFloatConversionError::Infinity => {
                PyOverflowError::new_err(reason.to_string())
            }
            _ => PyValueError::new_err(reason.to_string()),
        })
    } else {
        let numerator = try_py_integral_to_big_int(
            value
                .getattr(intern!(py, "numerator"))
                .map_err(|_| PyTypeError::new_err(INVALID_SCALAR_TYPE_ERROR_MESSAGE))?,
        )?;
        let denominator = try_py_integral_to_big_int(
            value
                .getattr(intern!(py, "denominator"))
                .map_err(|_| PyTypeError::new_err(INVALID_SCALAR_TYPE_ERROR_MESSAGE))?,
        )?;
        match Fraction::new(numerator, denominator) {
            Some(value) => Ok(value),
            None => Err(PyZeroDivisionError::new_err(
                UNDEFINED_DIVISION_ERROR_MESSAGE,
            )),
        }
    }
}

static mut ROOT_MODULE: Option<&PyModule> = None;

fn extract_from_sequence<'a, Wrapper: FromPyObject<'a>, Wrapped: From<Wrapper>>(
    sequence: &'a PySequence,
) -> PyResult<Vec<Wrapped>> {
    let mut result = Vec::<Wrapped>::with_capacity(sequence.len()?);
    for element in sequence.iter()? {
        result.push(element?.extract::<Wrapper>()?.into());
    }
    Ok(result)
}

#[pymodule]
fn _cexact(_py: Python, module: &PyModule) -> PyResult<()> {
    module.add_class::<PyExactContour>()?;
    module.add_class::<PyExactPoint>()?;
    module.add_class::<PyExactPolygon>()?;
    module.add_class::<PyExactSegment>()?;
    unsafe {
        let py = Python::assume_gil_acquired();
        ROOT_MODULE = Some(py.import("rene")?);
    }
    Ok(())
}

#[pymodule]
fn _crene(_py: Python, module: &PyModule) -> PyResult<()> {
    module.add_class::<PyOrientation>()?;
    module.add_class::<PyRelation>()?;
    module.add("MIN_CONTOUR_VERTICES_COUNT", MIN_CONTOUR_VERTICES_COUNT)?;
    Ok(())
}

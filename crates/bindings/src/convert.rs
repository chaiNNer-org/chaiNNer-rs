use image_core::{
    util::slice_as_chunks, FromFlat, Image, ImageView, IntoPixels, NDimCow, NDimImage, NDimView,
    Shape, ShapeMismatch, Size,
};
use numpy::{
    ndarray::{Array3, ArrayView2, ArrayView3},
    IntoPyArray, Ix3, PyArray3, PyReadonlyArrayDyn,
};
use pyo3::{exceptions::PyValueError, PyResult, Python};

pub fn get_channels(img: &PyReadonlyArrayDyn<f32>) -> usize {
    let data = img.shape();
    if data.len() >= 3 {
        data[2]
    } else {
        1
    }
}

fn py_to_shape(shape: &[usize]) -> Shape {
    if shape.len() == 2 {
        Shape::new(shape[1], shape[0], 1)
    } else {
        assert_eq!(shape.len(), 3);
        // python shape is in height width channels
        Shape::new(shape[1], shape[0], shape[2])
    }
}

fn new_numpy_array(size: Size, channels: usize, data: Vec<f32>) -> Array3<f32> {
    let shape = Ix3(size.height, size.width, channels);
    Array3::from_shape_vec(shape, data).expect("Expect creation of numpy array to succeed.")
}

fn view_numpy<'a>(ndarray: &'a PyReadonlyArrayDyn<'a, f32>) -> Option<NDimView<'a>> {
    let shape = py_to_shape(ndarray.shape());

    if ndarray.is_c_contiguous() {
        if let Ok(data) = ndarray.as_slice() {
            return Some(NDimView::new(shape, data));
        }
    }

    None
}
fn read_numpy<'a>(ndarray: &'a PyReadonlyArrayDyn<'a, f32>) -> NDimCow<'a> {
    if let Some(view) = view_numpy(ndarray) {
        return view.into();
    }

    let shape = py_to_shape(ndarray.shape());
    let array = ndarray.as_array();

    // https://github.com/rust-ndarray/ndarray/issues/1339
    let data = match array.shape().len() {
        2 => {
            let array: ArrayView2<f32> = array.into_dimensionality().unwrap();
            array.to_owned().into_raw_vec()
        }
        3 => {
            let array: ArrayView3<f32> = array.into_dimensionality().unwrap();
            array.to_owned().into_raw_vec()
        }
        _ => panic!("Invalid shape for image. Only 2D and 3D arrays are supported."),
    };

    NDimImage::new(shape, data).into()
}

pub trait IntoNumpy {
    fn into_numpy(self) -> Array3<f32>;
}

impl<T: Into<NDimImage>> IntoNumpy for T {
    fn into_numpy(self) -> Array3<f32> {
        let image: NDimImage = self.into();
        new_numpy_array(image.size(), image.channels(), image.take())
    }
}

pub trait IntoPy {
    fn into_py(self, py: Python<'_>) -> &PyArray3<f32>;
}
impl<T: IntoNumpy> IntoPy for T {
    fn into_py(self, py: Python<'_>) -> &PyArray3<f32> {
        self.into_numpy().into_pyarray(py)
    }
}

pub trait LoadImage<T> {
    fn load_image(self) -> PyResult<T>;
}
impl<'py> LoadImage<NDimCow<'py>> for &'py PyReadonlyArrayDyn<'py, f32> {
    fn load_image(self) -> PyResult<NDimCow<'py>> {
        Ok(read_numpy(self))
    }
}
impl<'py> LoadImage<NDimImage> for &'py PyReadonlyArrayDyn<'py, f32> {
    fn load_image(self) -> PyResult<NDimImage> {
        Ok(read_numpy(self).into_owned())
    }
}
impl<'py, T> LoadImage<Image<T>> for &'py PyReadonlyArrayDyn<'py, f32>
where
    T: FromFlat,
{
    fn load_image(self) -> PyResult<Image<T>> {
        let cow = read_numpy(self);
        let result = match cow {
            NDimCow::View(view) => view.into_pixels(),
            NDimCow::Image(image) => image.into_pixels(),
        };
        match result {
            Ok(image) => Ok(image),
            Err(ShapeMismatch { actual, expected }) => Err(PyValueError::new_err(format!(
                "Image does not have the right shape. Expected {} channel(s) but found {}.",
                expected
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
                actual
            ))),
        }
    }
}

pub trait ViewImage<T> {
    fn view_image(self) -> Option<T>;
}
impl<'py> ViewImage<NDimView<'py>> for &'py PyReadonlyArrayDyn<'py, f32> {
    fn view_image(self) -> Option<NDimView<'py>> {
        view_numpy(self)
    }
}
impl<'py> ViewImage<ImageView<'py, f32>> for &'py PyReadonlyArrayDyn<'py, f32> {
    fn view_image(self) -> Option<ImageView<'py, f32>> {
        if let Some(view) = view_numpy(self) {
            if view.channels() == 1 {
                let size = view.size();
                return Some(ImageView::new(size, view.data()));
            }
        }
        None
    }
}
impl<'py, const N: usize> ViewImage<ImageView<'py, [f32; N]>>
    for &'py PyReadonlyArrayDyn<'py, f32>
{
    fn view_image(self) -> Option<ImageView<'py, [f32; N]>> {
        if let Some(view) = view_numpy(self) {
            if view.channels() == N {
                let (chunks, rest) = slice_as_chunks(view.data());
                assert!(rest.is_empty());
                let size = view.size();
                return Some(ImageView::new(size, chunks));
            }
        }
        None
    }
}

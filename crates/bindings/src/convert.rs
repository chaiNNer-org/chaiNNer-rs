use image_core::{
    util::slice_as_chunks, FromFlat, Image, ImageView, IntoPixels, NDimCow, NDimImage, NDimView,
    Shape, ShapeMismatch, Size,
};
use numpy::{
    ndarray::{Array3, Dimension},
    IntoPyArray, Ix3, PyArray3, PyReadonlyArray, PyReadonlyArray2, PyReadonlyArray3,
};
use pyo3::{exceptions::PyValueError, FromPyObject, PyResult, Python};

#[derive(FromPyObject)]
pub enum PyImage<'py> {
    D2(PyReadonlyArray2<'py, f32>),
    D3(PyReadonlyArray3<'py, f32>),
}

impl PyImage<'_> {
    pub fn shape(&self) -> Shape {
        match self {
            PyImage::D2(img) => {
                let shape = img.shape();
                Shape::new(shape[1], shape[0], 1)
            }
            PyImage::D3(img) => {
                let shape = img.shape();
                Shape::new(shape[1], shape[0], shape[2])
            }
        }
    }
    pub fn channels(&self) -> usize {
        self.shape().channels
    }
    pub fn size(&self) -> Size {
        self.shape().size()
    }

    /// Tries to create a view of the image.
    ///
    /// This is only possible if the image is contiguous in memory and C-contiguous.
    pub fn try_view(&'_ self) -> Option<NDimView<'_>> {
        match self {
            PyImage::D2(img) => {
                if img.is_c_contiguous() {
                    if let Ok(data) = img.as_slice() {
                        return Some(NDimView::new(self.shape(), data));
                    }
                }
            }
            PyImage::D3(img) => {
                if img.is_c_contiguous() {
                    if let Ok(data) = img.as_slice() {
                        return Some(NDimView::new(self.shape(), data));
                    }
                }
            }
        };

        None
    }

    /// Creates a view of the image of possible. If not possible, it will copy
    /// the image into a Vec.
    pub fn as_contiguous(&'_ self) -> NDimCow<'_> {
        if let Some(view) = self.try_view() {
            return view.into();
        }

        let shape = self.shape();

        fn to_vec<D: Dimension>(img: &PyReadonlyArray<f32, D>) -> Vec<f32> {
            img.as_array().iter().copied().collect()
        }

        match self {
            PyImage::D2(img) => NDimImage::new(shape, to_vec(img)).into(),
            PyImage::D3(img) => NDimImage::new(shape, to_vec(img)).into(),
        }
    }
}

fn new_numpy_array(size: Size, channels: usize, data: Vec<f32>) -> Array3<f32> {
    let shape = Ix3(size.height, size.width, channels);
    Array3::from_shape_vec(shape, data).expect("Expect creation of numpy array to succeed.")
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
impl<'py> LoadImage<NDimCow<'py>> for &'py PyImage<'py> {
    fn load_image(self) -> PyResult<NDimCow<'py>> {
        Ok(self.as_contiguous())
    }
}
impl<'py> LoadImage<NDimImage> for &'py PyImage<'py> {
    fn load_image(self) -> PyResult<NDimImage> {
        Ok(self.as_contiguous().into_owned())
    }
}
impl<'py, T> LoadImage<Image<T>> for &'py PyImage<'py>
where
    T: FromFlat,
{
    fn load_image(self) -> PyResult<Image<T>> {
        let cow = self.as_contiguous();
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
impl<'py> ViewImage<NDimView<'py>> for &'py PyImage<'py> {
    fn view_image(self) -> Option<NDimView<'py>> {
        self.try_view()
    }
}
impl<'py> ViewImage<ImageView<'py, f32>> for &'py PyImage<'py> {
    fn view_image(self) -> Option<ImageView<'py, f32>> {
        if let Some(view) = self.try_view() {
            if view.channels() == 1 {
                let size = view.size();
                return Some(ImageView::new(size, view.data()));
            }
        }
        None
    }
}
impl<'py, const N: usize> ViewImage<ImageView<'py, [f32; N]>> for &'py PyImage<'py> {
    fn view_image(self) -> Option<ImageView<'py, [f32; N]>> {
        if let Some(view) = self.try_view() {
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

/// A macro for converting native numpy arrays to images.
///
/// The image type will automatically be inferred based on usage.
/// If the numpy array cannot be converted, an error will be early-returned.
#[macro_export]
macro_rules! load_image {
    ($img:ident) => {{
        use $crate::convert::{ToOwnedImage};
        match $img.to_owned_image() {
            Ok(r) => r,
            Err(e) => {
                return Err(PyValueError::new_err(format!(
                    "Argument '{}' does not have the right shape. Expected {} channel(s) but found {}.",
                    stringify!($img),
                    e.expected
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>()
                        .join(", "),
                    e.actual
                )))
            }
        }}
    };
}

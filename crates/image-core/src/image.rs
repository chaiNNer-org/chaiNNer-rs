use std::slice::ChunksExact;

/// A non-empty size consisting of width and height in that order.
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Size {
    pub width: usize,
    pub height: usize,
}

impl Size {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }
    pub fn scale(&self, factor: f64) -> Self {
        Self {
            width: (self.width as f64 * factor).ceil() as usize,
            height: (self.height as f64 * factor).ceil() as usize,
        }
    }

    pub fn len(&self) -> usize {
        self.width * self.height
    }

    /// Returns an iterator that goes through all positions of this size in row major order.
    pub fn iter_pos(&self) -> impl Iterator<Item = (usize, usize)> {
        let w = self.width;
        let h = self.height;
        (0..h).flat_map(move |y| (0..w).map(move |x| (x, y)))
    }
}

#[derive(Clone)]
pub struct Image<P> {
    data: Vec<P>,
    size: Size,
}

impl<P> Image<P> {
    pub fn new(size: Size, data: Vec<P>) -> Self {
        assert_eq!(size.len(), data.len());
        Self { data, size }
    }
    pub fn from_fn(size: Size, f: impl Fn(usize, usize) -> P) -> Self {
        let f = &f;

        let mut data = Vec::with_capacity(size.len());
        data.extend((0..size.height).flat_map(|y| (0..size.width).map(move |x| f(x, y))));

        Self::new(size, data)
    }

    pub fn size(&self) -> Size {
        self.size
    }

    pub fn width(&self) -> usize {
        self.size().width
    }
    pub fn height(&self) -> usize {
        self.size().height
    }
    pub fn len(&self) -> usize {
        self.size().len()
    }

    pub fn take(self) -> Vec<P> {
        self.data
    }

    /// The pixel data of the image.
    ///
    /// Pixel data is layed out in row-major order.
    pub fn data(&self) -> &[P] {
        self.data.as_slice()
    }

    pub fn rows<'a>(&'a self) -> ChunksExact<'a, P> {
        self.data().chunks_exact(self.width())
    }

    pub fn map<T>(&self, f: impl Fn(&P) -> T) -> Image<T> {
        Image {
            data: self.data().iter().map(f).collect(),
            size: self.size(),
        }
    }
    pub fn map_pos<T>(&self, f: impl Fn(&P, usize, usize) -> T) -> Image<T> {
        let f = &f;
        let data = self
            .rows()
            .enumerate()
            .flat_map(|(y, line)| line.iter().enumerate().map(move |(x, p)| f(p, x, y)))
            .collect();

        Image {
            data,
            size: self.size(),
        }
    }

    /// The pixel data of the image.
    ///
    /// Pixel data is layed out in row-major order.
    pub fn data_mut(&mut self) -> &mut [P] {
        self.data.as_mut_slice()
    }

    pub fn change<T>(&mut self, f: impl Fn(&P) -> P) {
        for p in self.data.iter_mut() {
            *p = f(p);
        }
    }
}
impl<P> Image<P>
where
    P: Clone,
{
    pub fn from_const(size: Size, constant: P) -> Self {
        Self {
            data: vec![constant; size.len()],
            size,
        }
    }

    pub fn fill(&mut self, c: P) {
        self.data.fill(c);
    }
}

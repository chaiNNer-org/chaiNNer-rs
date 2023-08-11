use std::{
    fs::{create_dir_all, File},
    io::{Cursor, Read, Write},
    path::Path,
};

use glam::{Vec3, Vec3A, Vec4};
use image::RgbaImage;
use image_core::{
    util::{slice_as_chunks, slice_as_chunks_mut},
    Image, NDimImage,
};

trait ToRGBA {
    fn to_rgba(&self) -> [u8; 4];
}

impl ToRGBA for Vec4 {
    fn to_rgba(&self) -> [u8; 4] {
        let c = ((*self) * 255.).round();
        [c.x as u8, c.y as u8, c.z as u8, c.w as u8]
    }
}
impl ToRGBA for Vec3 {
    fn to_rgba(&self) -> [u8; 4] {
        let c = ((*self) * 255.).round();
        [c.x as u8, c.y as u8, c.z as u8, 255]
    }
}
impl ToRGBA for Vec3A {
    fn to_rgba(&self) -> [u8; 4] {
        let c = ((*self) * 255.).round();
        [c.x as u8, c.y as u8, c.z as u8, 255]
    }
}
impl ToRGBA for f32 {
    fn to_rgba(&self) -> [u8; 4] {
        let c = ((*self) * 255.).round() as u8;
        [c, c, c, 255]
    }
}
impl ToRGBA for [f32; 1] {
    fn to_rgba(&self) -> [u8; 4] {
        let c = (self[0] * 255.).round() as u8;
        [c, c, c, 255]
    }
}
impl ToRGBA for [f32; 3] {
    fn to_rgba(&self) -> [u8; 4] {
        [
            (self[0] * 255.).round() as u8,
            (self[1] * 255.).round() as u8,
            (self[2] * 255.).round() as u8,
            255,
        ]
    }
}
impl ToRGBA for [f32; 4] {
    fn to_rgba(&self) -> [u8; 4] {
        [
            (self[0] * 255.).round() as u8,
            (self[1] * 255.).round() as u8,
            (self[2] * 255.).round() as u8,
            (self[3] * 255.).round() as u8,
        ]
    }
}

pub trait ImageSnapshot {
    fn snapshot(&self, snap_id: &str);
}

impl<P: ToRGBA> ImageSnapshot for Image<P> {
    fn snapshot(&self, snap_id: &str) {
        compare_snapshot(new_rgba(self), snap_id)
    }
}
impl ImageSnapshot for NDimImage {
    fn snapshot(&self, snap_id: &str) {
        let rgba = match self.channels() {
            1 => {
                let data: Vec<f32> = self.data().to_vec();
                new_rgba(&Image::new(self.size(), data))
            }
            3 => {
                let (chucks, rest) = slice_as_chunks(self.data());
                assert!(rest.is_empty());
                let data: Vec<[f32; 3]> = chucks.to_vec();
                new_rgba(&Image::new(self.size(), data))
            }
            4 => {
                let (chucks, rest) = slice_as_chunks(self.data());
                assert!(rest.is_empty());
                let data: Vec<[f32; 4]> = chucks.to_vec();
                new_rgba(&Image::new(self.size(), data))
            }
            _ => panic!("Unsupported channel count"),
        };

        compare_snapshot(rgba, snap_id)
    }
}

fn new_rgba<P: ToRGBA>(image: &Image<P>) -> RgbaImage {
    let mut rgba = RgbaImage::new(image.width() as u32, image.height() as u32);
    let (chunks, rest) = slice_as_chunks_mut::<_, 4>(&mut rgba);
    assert!(rest.is_empty());
    let image_data = image.data();
    assert_eq!(chunks.len(), image_data.len());
    for (p, i) in chunks.iter_mut().zip(image_data) {
        *p = i.to_rgba();
    }
    rgba
}

fn get_png_bytes(image: &RgbaImage) -> Vec<u8> {
    let mut bytes = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut bytes), image::ImageOutputFormat::Png)
        .unwrap();
    bytes
}

const SNAP_DIR: &str = "__snapshots/";

fn get_snap_path(snap_id: &str) -> String {
    SNAP_DIR.to_owned() + snap_id + ".png"
}

fn read_snap(snap_id: &str) -> Option<Vec<u8>> {
    let snap_path = get_snap_path(snap_id);
    let file = File::open(&snap_path);
    let mut file = match file {
        Ok(file) => file,
        Err(_) => {
            if !Path::new(&snap_path).exists() {
                return None;
            }
            unreachable!("Unable to read snapshot for {}", snap_id)
        }
    };

    let mut snap_bytes = Vec::new();
    file.read_to_end(&mut snap_bytes).unwrap();
    Some(snap_bytes)
}

fn write_snap(snap_id: &str, data: &[u8]) {
    let snap_path = get_snap_path(snap_id);
    create_dir_all(SNAP_DIR).unwrap();
    let mut file = File::create(snap_path).unwrap();
    file.write_all(data).unwrap();
}

fn compare_snapshot(image: RgbaImage, snap_id: &str) {
    let bytes = get_png_bytes(&image);

    if let Some(snap_bytes) = read_snap(snap_id) {
        let unchanged = snap_bytes == bytes;
        if cfg!(feature = "snap-write") {
            if !unchanged {
                write_snap(snap_id, &bytes);
            }
        } else {
            assert!(unchanged, "The result for {} changed.", snap_id);
        }
    } else {
        write_snap(snap_id, &bytes);
    }
}

use std::{
    fs::{create_dir_all, File},
    io::{Cursor, Read, Write},
    path::Path,
};

use glam::{Vec3, Vec3A, Vec4};
use image::RgbaImage;
use image_core::Image;

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

fn new_rgba<P: ToRGBA>(image: &Image<P>) -> RgbaImage {
    let mut rgba = RgbaImage::new(image.width() as u32, image.height() as u32);
    let (chunks, rest) = rgba.as_chunks_mut::<4>();
    assert!(rest.is_empty());
    let image_data = image.data();
    assert_eq!(chunks.len(), image_data.len());
    for (p, i) in chunks.into_iter().zip(image_data) {
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

fn read_snap(snap_id: &str) -> Vec<u8> {
    let snap_path = get_snap_path(snap_id);
    let file = File::open(&snap_path);
    let mut file = match file {
        Ok(file) => file,
        Err(_) => {
            if Path::new(&snap_path).exists() {
                unreachable!("No snapshot for {}", snap_id)
            }
            unreachable!("Unable to read snapshot for {}", snap_id)
        }
    };

    let mut snap_bytes = Vec::new();
    file.read_to_end(&mut snap_bytes).unwrap();
    snap_bytes
}

fn compare_snapshot(image: RgbaImage, snap_id: &str) {
    let bytes = get_png_bytes(&image);

    let snap_path = get_snap_path(snap_id);

    if cfg!(feature = "snap-write") {
        if Path::new(&snap_path).exists() {
            let snap_bytes = read_snap(snap_id);
            if snap_bytes == bytes {
                // snap is still up to date
                return;
            }
        } else {
            create_dir_all(SNAP_DIR).unwrap();
        }

        let mut file = File::create(snap_path).unwrap();
        file.write_all(&bytes).unwrap();
    } else {
        let snap_bytes = read_snap(snap_id);
        let unchanged = snap_bytes == bytes;
        assert!(unchanged, "The result for {} changed.", snap_id);
    }
}

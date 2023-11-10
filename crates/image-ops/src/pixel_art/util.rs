#[inline]
pub fn write_2x<T>(dest: &mut [T], w: usize, x: usize, y: usize, [r1, r2, r3, r4]: [T; 4]) {
    let y2 = y * 2;
    let x2 = x * 2;
    let w2 = w * 2;
    dest[y2 * w2 + x2] = r1;
    dest[y2 * w2 + x2 + 1] = r2;
    dest[(y2 + 1) * w2 + x2] = r3;
    dest[(y2 + 1) * w2 + x2 + 1] = r4;
}

#[inline]
pub fn write_3x<T>(
    dest: &mut [T],
    w: usize,
    x: usize,
    y: usize,
    [r1, r2, r3, r4, r5, r6, r7, r8, r9]: [T; 9],
) {
    let y3 = y * 3;
    let x3 = x * 3;
    let w3 = w * 3;
    dest[y3 * w3 + x3] = r1;
    dest[y3 * w3 + x3 + 1] = r2;
    dest[y3 * w3 + x3 + 2] = r3;
    dest[(y3 + 1) * w3 + x3] = r4;
    dest[(y3 + 1) * w3 + x3 + 1] = r5;
    dest[(y3 + 1) * w3 + x3 + 2] = r6;
    dest[(y3 + 2) * w3 + x3] = r7;
    dest[(y3 + 2) * w3 + x3 + 1] = r8;
    dest[(y3 + 2) * w3 + x3 + 2] = r9;
}

#[inline]
pub fn write_4x<T: Copy>(dest: &mut [T], w: usize, x: usize, y: usize, buffer: [T; 16]) {
    let y4 = y * 4;
    let x4 = x * 4;
    let w4 = w * 4;
    for y in 0..4 {
        dest[(y4 + y) * w4 + x4] = buffer[y * 4];
        dest[(y4 + y) * w4 + x4 + 1] = buffer[y * 4 + 1];
        dest[(y4 + y) * w4 + x4 + 2] = buffer[y * 4 + 2];
        dest[(y4 + y) * w4 + x4 + 3] = buffer[y * 4 + 3];
    }
}

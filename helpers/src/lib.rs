#[cfg(test)]
mod tests {

    use crate::*;

    #[test]
    fn double_width_doubles() {
        let initial = vec![1u8,2u8,3u8,4u8];
        let expected = vec![1u8,1u8,2u8,2u8,3u8,3u8,4u8,4u8];
        assert_eq!(double_width(&initial), expected);
    }

    #[test]
    fn greyscale_works() {
        let initial = vec![0u8,15u8];
        let expected = vec![0u8,0u8,0u8,255u8,240u8,240u8,240u8,255u8];
        assert_eq!(conv_greyscale(&initial), expected);
    }
    
    #[test]
    fn rgba_works() {
        let initial = vec![0u8,14u8];
        let expected = vec![0u8,0u8,0u8,255u8,255u8,255u8,127u8,255u8];
        assert_eq!(conv_rgba(&initial), expected);
    }
    
    #[test]
    fn rgba_trans0_works() {
        let initial = vec![0u8,14u8];
        let expected = vec![0u8,0u8,0u8,0u8,255u8,255u8,127u8,255u8];
        assert_eq!(conv_rgba_transparent(&initial,0), expected);
    }
    
    #[test]
    fn rgba_trans2_works() {
        let initial = vec![1u8,2u8];
        let expected = vec![0u8,0u8,171u8,255u8,0u8,171u8,0u8,0u8];
        assert_eq!(conv_rgba_transparent(&initial,2), expected);
    }

}


pub fn double_pic_width(data:&[u8]) -> Vec<u8> {
    let mut out_vec = Vec::new();
    out_vec.reserve(data.len()*2);
    for a in data {
        out_vec.push(*a);
        out_vec.push(*a);
    }
    return out_vec;
}

pub fn double_width(data:&Vec<u8>) -> Vec<u8> {
    let mut out_vec = Vec::new();
    out_vec.reserve(data.len()*2);
    for a in data {
        out_vec.push(*a);
        out_vec.push(*a);
    }
    return out_vec;
}

pub fn conv_greyscale(data: &Vec<u8>) -> Vec<u8> {
    let mut out_vec = Vec::new();
    out_vec.reserve(data.len()*4);
    for a in data {
        out_vec.push(a<<4);
        out_vec.push(a<<4);
        out_vec.push(a<<4);
        out_vec.push(255);
    }

    return out_vec;
}

pub fn conv_rgba(data: &Vec<u8>) -> Vec<u8> {
    let mut out_vec = Vec::new();
    out_vec.reserve(data.len()*4);
    for a in data {
        out_vec.extend_from_slice(&conv_colour(a,false));
    }

    return out_vec;
}

fn conv_colour(a: &u8, trans:bool) -> [u8;4] {
    return match a {
        15..=255 => [255u8,255,255,if trans {0} else {255}],
              14 => [255u8,255,127,if trans {0} else {255}],
              13 => [255u8,127,255,if trans {0} else {255}],
              12 => [255u8,127,127,if trans {0} else {255}],
              11 => [127u8,255,255,if trans {0} else {255}],
              10 => [127u8,255,127,if trans {0} else {255}],
               9 => [127u8,127,255,if trans {0} else {255}],
               8 => [127u8,127,127,if trans {0} else {255}],
               7 => [171u8,171,171,if trans {0} else {255}],
               6 => [171u8,127,  0,if trans {0} else {255}],
               5 => [171u8,  0,171,if trans {0} else {255}],
               4 => [171u8,  0,  0,if trans {0} else {255}],
               3 => [  0u8,171,171,if trans {0} else {255}],
               2 => [  0u8,171,  0,if trans {0} else {255}],
               1 => [  0u8,  0,171,if trans {0} else {255}],
               0 => [  0u8,  0,  0,if trans {0} else {255}],
    }
}

pub fn conv_rgba_transparent(data: &Vec<u8>, trans_col:u8) -> Vec<u8> {
    let mut out_vec = Vec::new();
    out_vec.reserve(data.len()*4);
    for a in data {
        out_vec.extend_from_slice(&conv_colour(a,*a==trans_col));
    }

    return out_vec;
}

use std::fs::{File, self};
use std::path::Path;
use std::io::BufWriter;

pub fn dump_png(filepath: &str, width:u32, height:u32, data: &Vec<u8>) {
    let path = Path::new(filepath);
    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_trns(vec!(0xFFu8, 0xFFu8, 0xFFu8, 0xFFu8));
    encoder.set_source_gamma(png::ScaledFloat::from_scaled(45455)); // 1.0 / 2.2, scaled by 100000
    encoder.set_source_gamma(png::ScaledFloat::new(1.0 / 2.2));     // 1.0 / 2.2, unscaled, but rounded
    let source_chromaticities = png::SourceChromaticities::new(     // Using unscaled instantiation here
        (0.31270, 0.32900),
        (0.64000, 0.33000),
        (0.30000, 0.60000),
        (0.15000, 0.06000)
    );
    encoder.set_source_chromaticities(source_chromaticities);
    let mut writer = encoder.write_header().unwrap();

    writer.write_image_data(data).unwrap(); // Save
}

pub struct Root<'a> {
    base_path:&'a Path,
}

impl<'a> Root<'_> {
    pub fn new(base_path:&'a str) -> Root {
        Root {base_path:Path::new(base_path)}
    }

    pub fn read_data_or_default(&self,file:&str) -> Vec<u8> {
        return fs::read(self.base_path.join(file).into_os_string()).unwrap_or_default();
    }
}

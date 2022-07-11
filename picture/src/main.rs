use std::fs::{self, File};

use dir_resource::ResourceDirectory;
use volume::Volume;

fn main() {
    let bytes = fs::read("../images/Space Quest- The Sarien Encounter v1.0X (1986)(Sierra On-Line, Inc.) [Adventure]/PICDIR").unwrap_or_default();

    let dir = ResourceDirectory::new(bytes.into_iter()).unwrap();

    let bytes = fs::read("../images/Space Quest- The Sarien Encounter v1.0X (1986)(Sierra On-Line, Inc.) [Adventure]/VOL.1").unwrap_or_default();

    let volume1 = Volume::new(bytes.into_iter()).unwrap();

    for a in dir {
        println!("V{} P{}",a.volume,a.position);
    }

    let doubled_width = double_width(&vec![15;160*200]);

    let rgba = conv_rgba(&doubled_width);

    dump_png(r"../test.png",320,200,&rgba);
}

fn double_width(data:&Vec<u8>) -> Vec<u8> {
    let mut out_vec = Vec::new();
    out_vec.reserve(data.len()*2);
    for a in data {
        out_vec.push(*a);
        out_vec.push(*a);
    }
    return out_vec;
}

fn conv_rgba(data: &Vec<u8>) -> Vec<u8> {
    let mut out_vec = Vec::new();
    out_vec.reserve(data.len()*4);
    for a in data {
        match a {
            15..=255 => out_vec.extend_from_slice(&[255u8,255,255,255]),
                  14 => out_vec.extend_from_slice(&[255u8,255,127,255]),
                  13 => out_vec.extend_from_slice(&[255u8,127,255,255]),
                  12 => out_vec.extend_from_slice(&[255u8,127,127,255]),
                  11 => out_vec.extend_from_slice(&[127u8,255,255,255]),
                  10 => out_vec.extend_from_slice(&[127u8,255,127,255]),
                   9 => out_vec.extend_from_slice(&[127u8,127,255,255]),
                   8 => out_vec.extend_from_slice(&[127u8,127,127,255]),
                   7 => out_vec.extend_from_slice(&[171u8,171,171,255]),
                   6 => out_vec.extend_from_slice(&[171u8,127,  0,255]),
                   5 => out_vec.extend_from_slice(&[171u8,  0,171,255]),
                   4 => out_vec.extend_from_slice(&[171u8,  0,  0,255]),
                   3 => out_vec.extend_from_slice(&[  0u8,171,171,255]),
                   2 => out_vec.extend_from_slice(&[  0u8,171,  0,255]),
                   1 => out_vec.extend_from_slice(&[  0u8,  0,171,255]),
                   0 => out_vec.extend_from_slice(&[  0u8,  0,  0,255]),
        }
    }

    return out_vec;
}

use std::path::Path;
use std::io::BufWriter;

fn dump_png(filepath: &str, width:u32, height:u32, data: &Vec<u8>) {
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

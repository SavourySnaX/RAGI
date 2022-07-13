
use std::fs::{self, File};
use std::{path::Path, vec};

use dir_resource::{ResourceDirectory, ResourceDirectoryEntry};
use volume::Volume;

struct Root<'a> {
    base_path:&'a Path,
}

impl<'a> Root<'_> {
    pub fn new(base_path:&'a str) -> Root {
        Root {base_path:Path::new(base_path)}
    }
}

fn main() {

    //let root = Root::new("../images/Leisure Suit Larry in the Land of the Lounge Lizards (1987)(Sierra On-Line, Inc.) [Adventure]/");
    let root = Root::new("../images/Space Quest- The Sarien Encounter v1.0X (1986)(Sierra On-Line, Inc.) [Adventure]/");

    let bytes = fs::read(root.base_path.join("VIEWDIR").into_os_string()).unwrap_or_default();

    let dir = ResourceDirectory::new(bytes.into_iter()).unwrap();

    for (index,entry) in dir.into_iter().enumerate() {
        if !entry.empty() {
            println!("{} : V{} P{}",index,entry.volume,entry.position);
            dump_view_resource(&root,&entry, index);
        }
    }

}

fn dump_view_resource(root:&Root,entry:&ResourceDirectoryEntry, index:usize) {

    let bytes = fs::read(root.base_path.join(format!("VOL.{}", entry.volume)).into_os_string()).unwrap_or_default();

    let volume = Volume::new(bytes.into_iter()).unwrap();

    let data = fetch_data_slice(&volume, entry);

    fs::write(format!("../{}-binary.bin",index).as_str(),data).unwrap();

    let view = match process_view(index, &volume, entry) {
        Ok(b) => b,
        Err(s) => panic!("Failed due to : {}", s),
    };

}

//todo upgrade to Result
fn fetch_data_slice<'a>(volume: &'a Volume, entry: &ResourceDirectoryEntry) -> &'a [u8] {

    let slice = &volume.data[entry.position as usize..];
    let slice = &slice[3..]; // Skip 0x1234 + Vol

    let length:usize = slice[0].into();
    let upper:usize = slice[1].into();
    let upper = upper<<8;
    let length = length+upper;
    &slice[2..length+2]
}

fn process_view(index:usize, volume:&Volume, entry: &ResourceDirectoryEntry) -> Result<String, String> {

    let slice = fetch_data_slice(volume, entry);
    let slice_iter = slice.iter();

    // Read in header (skip first 2 bytes as they are unknown)
    let mut slice_iter = slice_iter.skip(2);

    let loops = slice_iter.next().unwrap();
    let lsb_pos = slice_iter.next().unwrap();
    let msb_pos = slice_iter.next().unwrap();
    let position:usize = *msb_pos as usize;
    let position = position<<8;
    let position = position + (*lsb_pos as usize);
    let description_position = position;

    let mut loop_positions:Vec<usize> = Vec::new();
    loop_positions.reserve((*loops).into());
    for _i in 0..*loops {
        let lsb_pos = slice_iter.next().unwrap();
        let msb_pos = slice_iter.next().unwrap();
        let position:usize = *msb_pos as usize;
        let position = position<<8;
        let position = position + (*lsb_pos as usize);
        loop_positions.push(position);
    }

    if description_position!=0 {
        let slice = &slice[description_position..];
        let mut iter = slice.iter();
        let mut string = String::new();

        loop {
            let b = iter.next().unwrap();
            if *b==0 {
                break;
            }
            string = string + &String::from((*b) as char);
        }

        println!("Description : {}", string);
    }

    for (l_index,l) in loop_positions.into_iter().enumerate() {

        let slice = &slice[l..];
        let mut iter = slice.iter();

        // Read in loop header
        let cells = iter.next().unwrap();

        let mut cell_positions:Vec<usize> = Vec::new();
        cell_positions.reserve((*cells).into());
        for _c in 0..*cells {
            let lsb_pos = iter.next().unwrap();
            let msb_pos = iter.next().unwrap();
            let position:usize = *msb_pos as usize;
            let position = position<<8;
            let position = position + (*lsb_pos as usize);
            cell_positions.push(position);
        }

        for (c_index,c) in cell_positions.into_iter().enumerate() {

            let slice = &slice[c..];

            let mut iter = slice.iter();

            let width = iter.next().unwrap();
            let height = iter.next().unwrap();
            let flags = iter.next().unwrap();

            println!("W {} H {} Flags {:02X}",*width,*height,*flags);

            let trans_col = flags&0xF;
            let size:usize = (*width).into();
            let size = size * (*height as usize);

            // unpack our data 

            let mut image:Vec<u8> = vec![trans_col; size];
            let image_pos:usize = 0;

            for y in 0..*height {

                let mut pos = image_pos;
                pos += ((*width) as usize) * (y as usize);
                let mut rle = iter.next().unwrap();
                while *rle!=0 {
                    let color = (*rle)>>4;
                    let len = (*rle)&0xF;
                    for _p in 0..len {
                        image[pos]=color;
                        pos+=1;
                    }
                    rle = iter.next().unwrap();
                }
            }

            // Temporary
            let doubled_width = double_width(&image);

            let rgba = conv_rgba_transparent(&doubled_width, trans_col);

            dump_png(format!("../{}-cell-{}-{}.png",index, l_index, c_index).as_str(),(*width as u32)*2,*height as u32,&rgba);



        }
    }

    return Ok(String::from("POOP"));
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

fn conv_rgba_transparent(data: &Vec<u8>, trans_col:u8) -> Vec<u8> {
    let mut out_vec = Vec::new();
    out_vec.reserve(data.len()*4);
    for a in data {
        match a {
            15..=255 => out_vec.extend_from_slice(&[255u8,255,255,if *a==trans_col {0} else {255}]),
                  14 => out_vec.extend_from_slice(&[255u8,255,127,if *a==trans_col {0} else {255}]),
                  13 => out_vec.extend_from_slice(&[255u8,127,255,if *a==trans_col {0} else {255}]),
                  12 => out_vec.extend_from_slice(&[255u8,127,127,if *a==trans_col {0} else {255}]),
                  11 => out_vec.extend_from_slice(&[127u8,255,255,if *a==trans_col {0} else {255}]),
                  10 => out_vec.extend_from_slice(&[127u8,255,127,if *a==trans_col {0} else {255}]),
                   9 => out_vec.extend_from_slice(&[127u8,127,255,if *a==trans_col {0} else {255}]),
                   8 => out_vec.extend_from_slice(&[127u8,127,127,if *a==trans_col {0} else {255}]),
                   7 => out_vec.extend_from_slice(&[171u8,171,171,if *a==trans_col {0} else {255}]),
                   6 => out_vec.extend_from_slice(&[171u8,127,  0,if *a==trans_col {0} else {255}]),
                   5 => out_vec.extend_from_slice(&[171u8,  0,171,if *a==trans_col {0} else {255}]),
                   4 => out_vec.extend_from_slice(&[171u8,  0,  0,if *a==trans_col {0} else {255}]),
                   3 => out_vec.extend_from_slice(&[  0u8,171,171,if *a==trans_col {0} else {255}]),
                   2 => out_vec.extend_from_slice(&[  0u8,171,  0,if *a==trans_col {0} else {255}]),
                   1 => out_vec.extend_from_slice(&[  0u8,  0,171,if *a==trans_col {0} else {255}]),
                   0 => out_vec.extend_from_slice(&[  0u8,  0,  0,if *a==trans_col {0} else {255}]),
        }
    }

    return out_vec;
}

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


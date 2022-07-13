use std::collections::VecDeque;
use std::fs::{self, File};
use std::iter::Peekable;
use std::vec;

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

    let root = Root::new("../images/Leisure Suit Larry in the Land of the Lounge Lizards (1987)(Sierra On-Line, Inc.) [Adventure]/");
    //let root = Root::new("../images/Space Quest- The Sarien Encounter v1.0X (1986)(Sierra On-Line, Inc.) [Adventure]/");

    let bytes = fs::read(root.base_path.join("PICDIR").into_os_string()).unwrap_or_default();

    let dir = ResourceDirectory::new(bytes.into_iter()).unwrap();

    for (index,entry) in dir.into_iter().enumerate() {
        if !entry.empty() {
            println!("{} : V{} P{}",index,entry.volume,entry.position);
            dump_picture_resource(&root,&entry, index);
        }
    }

}

const WIDTH:i16 = 160;
const HEIGHT:i16 = 168;

fn dump_picture_resource(root:&Root,entry:&ResourceDirectoryEntry, index:usize) {

    let bytes = fs::read(root.base_path.join(format!("VOL.{}", entry.volume)).into_os_string()).unwrap_or_default();

    let volume = Volume::new(bytes.into_iter()).unwrap();

    let picture = match process_picture(index, false, &volume,entry) {
        Ok(b) => b,
        Err(s) => panic!("Failed due to : {}", s),
    };
    
    let doubled_width = double_width(&&picture.picture);

    let rgba = conv_rgba(&doubled_width);

    dump_png(format!("../{}-picture.png",index).as_str(),(WIDTH*2) as u32,HEIGHT as u32,&rgba);

    let doubled_width = double_width(&&&picture.priority);

    let rgba = conv_greyscale(&doubled_width);

    dump_png(format!("../{}-priority.png",index).as_str(),(WIDTH*2) as u32,HEIGHT as u32,&rgba);

    let volume_iter = fetch_data_iterator(&volume, entry);

    let data:Vec<u8> = volume_iter.cloned().collect();
    fs::write(format!("../{}-binary.bin",index).as_str(),data).unwrap();
}

struct PictureResource
{
    picture: Vec<u8>,
    priority: Vec<u8>,
}
//todo upgrade to Result
fn fetch_data_iterator<'a>(volume: &'a Volume, entry: &ResourceDirectoryEntry) -> impl Iterator<Item = &'a u8> {
    let volume_iter = volume.data.iter().skip(entry.position as usize);
    let mut volume_iter = volume_iter.skip(3);
    // Skip 0x1234 and vol
    let length:u16 = (*volume_iter.next().unwrap()).into();
    let upper:u16 = (*volume_iter.next().unwrap()).into();
    let upper = upper<<8;
    let length=length+upper;
    volume_iter.take(length as usize)
}


// Attach to volume manager - get picture resource...?
fn process_picture(index:usize, dump_parts:bool, volume:&Volume, entry: &ResourceDirectoryEntry) -> Result<PictureResource, String> {

    let mut picture = vec![15u8;(WIDTH*HEIGHT) as usize];
    let mut priority = vec![4u8;(WIDTH*HEIGHT) as usize];

    let mut volume_iter = fetch_data_iterator(volume, entry).peekable();

    let mut colour_pen=15u8;
    let mut priority_pen=4u8;
    let mut colour_on=false;
    let mut priority_on=false;

    let mut last_match=0u8;
    let mut part_index = 0;
    let mut plot_pen_size:u8 = 0;
    let mut plot_pen_splatter=false;
    let mut plot_pen_rectangle:bool=true;

    while let Some(b) = volume_iter.next() {
        match b {
            0xF0 => { colour_on=true; colour_pen=*volume_iter.next().unwrap(); println!("Change picture color, color pen down : {}",colour_pen) },
            0xF1 => { colour_on=false; println!("Color pen up"); },
            0xF2 => { priority_on=true; priority_pen=*volume_iter.next().unwrap(); println!("Change priority color, priority pen down : {}",priority_pen); },
            0xF3 => { priority_on=false; println!("Priority pen up"); },
            0xF4 => { alternate_line(&mut picture,&mut priority,colour_on,priority_on,colour_pen,priority_pen,&mut volume_iter, false); }
            0xF5 => { alternate_line(&mut picture,&mut priority,colour_on,priority_on,colour_pen,priority_pen,&mut volume_iter, true); }
            0xF6 => { absolute_line(&mut picture,&mut priority,colour_on,priority_on,colour_pen,priority_pen,&mut volume_iter); },
            0xF7 => { relative_line(&mut picture,&mut priority,colour_on,priority_on,colour_pen,priority_pen,&mut volume_iter); },
            0xF8 => { fill(&mut picture,&mut priority,colour_on,priority_on,colour_pen,priority_pen,&mut volume_iter); },
            0xFA => { plot_pen(&mut picture,&mut priority,colour_on,priority_on,colour_pen,priority_pen,plot_pen_size,plot_pen_splatter,plot_pen_rectangle,&mut volume_iter); },

            0xFF => break,
            _ => return Err(format!("Unhandled control code {:02X}",b)),
        }

        if *b != last_match && dump_parts {

            let doubled_width = double_width(&picture);

            let rgba = conv_rgba(&doubled_width);

            dump_png(format!("../{}-picture-{}.png",index,part_index).as_str(),(WIDTH*2) as u32,HEIGHT as u32,&rgba);

            let doubled_width = double_width(&priority);

            let rgba = conv_greyscale(&doubled_width);

            dump_png(format!("../{}-priority-{}.png",index,part_index).as_str(),(WIDTH*2) as u32,HEIGHT as u32,&rgba);

            part_index = part_index+1;
            last_match=*b;
        }
    }

    return Ok(PictureResource {picture,priority});
}

fn rasterise_plot(picture:&mut Vec<u8>,priority:&mut Vec<u8>,colour_on:bool,priority_on:bool,colour_pen:u8,priority_pen:u8, x:i16, y:i16) {

    if x<0 || x>(WIDTH-1) || y<0 || y>(HEIGHT-1) {
        return;
    }

    let coord = (WIDTH*y+x) as usize;
    if colour_on {
        picture[coord]=colour_pen;
    }
    if priority_on {
        priority[coord]=priority_pen;
    }

}

fn rasterise_plot_pen(picture:&mut Vec<u8>,priority:&mut Vec<u8>,colour_on:bool,priority_on:bool,colour_pen:u8,priority_pen:u8,plot_pen_size:u8,plot_pen_splatter:bool,plot_pen_rectangle:bool,x:i16,y:i16) {

    // pen size 0-7
    if plot_pen_size != 0 {
        panic!("Pen Sizes > not supported");
    }

    if plot_pen_splatter {
        panic!("Pen Splatter not supported");
    }

    if !plot_pen_rectangle {
        panic!("Circle not supported");
    }

    rasterise_plot(picture, priority, colour_on, priority_on, colour_pen, priority_pen, x, y);
}


fn rasterise_line(picture:&mut Vec<u8>,priority:&mut Vec<u8>,colour_on:bool,priority_on:bool,colour_pen:u8,priority_pen:u8, x0:i16, y0:i16, x1:i16, y1:i16) {

    let dx = (x1-x0).abs();
    let sx = if x0<x1 {1i16} else {-1i16};
    let dy = -(y1-y0).abs();
    let sy = if y0<y1 {1i16} else {-1i16};
    let mut error = dx + dy;

    let mut x=x0;
    let mut y=y0;

    loop {

        rasterise_plot(picture, priority, colour_on, priority_on, colour_pen, priority_pen, x, y);

        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * error;
        if e2 >= dy {
            if x == x1 {
                break;
            }
            error = error + dy;
            x = x + sx;
        }
        if e2 <= dx {
            if y == y1 {
                break;
            }
            error = error + dx;
            y = y + sy;
        }
    }

}

fn rasterise_fill(picture:&mut Vec<u8>,priority:&mut Vec<u8>,colour_on:bool,priority_on:bool,colour_pen:u8,priority_pen:u8, x:u8, y:u8) {

    let mut queue:VecDeque<(u8,u8)> = VecDeque::new();

    if x>159 || y>199 {
        return;
    }

    queue.push_back((x,y));

    while !queue.is_empty() {

        let (x,y) = queue.pop_front().unwrap();

        let vec_coord = y as usize;
        let vec_coord = vec_coord * (WIDTH as usize);
        let vec_coord: usize = vec_coord + x as usize;

        if colour_on && picture[vec_coord]!=15 {
            continue;
        }

        if priority_on && !colour_on && priority[vec_coord]!=4 {
            continue;
        }

        if colour_on {
            picture[vec_coord]=colour_pen;
        }
        if priority_on {
            priority[vec_coord]=priority_pen;
        }

        if x<((WIDTH-1) as u8)  { queue.push_back((x+1,y)); }
        if x>0                  { queue.push_back((x-1,y)); }
        if y<((HEIGHT-1) as u8) { queue.push_back((x,y+1)); }
        if y>0                  { queue.push_back((x,y-1)); }
    }

}

fn alternate_line<'a, I>(picture:&mut Vec<u8>,priority:&mut Vec<u8>,colour_on:bool,priority_on:bool,colour_pen:u8,priority_pen:u8,volume_iter:&mut Peekable<I>, startx:bool)
where I: Iterator<Item = &'a u8> {

    let mut x0 = volume_iter.next().unwrap();
    let mut y0 = volume_iter.next().unwrap();

    let mut x=startx;
    let mut x1= x0;
    let mut y1= y0;

    rasterise_plot(picture, priority, colour_on, priority_on, colour_pen, priority_pen, (*x0).into(), (*y0).into());

    while let Some(b) = volume_iter.peek() {
        if **b >= 0xF0 {
            return;
        }

        let n = volume_iter.next().unwrap();

        if x {
            x1 = n;
        } else {
            y1 = n;
        }
        x=!x;

        println!("Alternating Line : {} {},{} -> {},{}",n,x0,y0,x1,y1);

        rasterise_line(picture, priority, colour_on, priority_on, colour_pen, priority_pen, (*x0).into(), (*y0).into(), (*x1).into(), (*y1).into());

        x0=x1;
        y0=y1;
    }
}


fn absolute_line<'a, I>(picture:&mut Vec<u8>,priority:&mut Vec<u8>,colour_on:bool,priority_on:bool,colour_pen:u8,priority_pen:u8,volume_iter:&mut Peekable<I>)
where I: Iterator<Item = &'a u8> {

    let mut x0 = volume_iter.next().unwrap();
    let mut y0 = volume_iter.next().unwrap();

    rasterise_plot(picture, priority, colour_on, priority_on, colour_pen, priority_pen, (*x0).into(), (*y0).into());

    while let Some(b) = volume_iter.peek() {
        if **b >= 0xF0 {
            return;
        }
        let x1 = volume_iter.next().unwrap();
        let y1 = volume_iter.next().unwrap();

        println!("Absolute Line : {},{} -> {},{}",x0,y0,x1,y1);

        rasterise_line(picture, priority, colour_on, priority_on, colour_pen, priority_pen, (*x0).into(), (*y0).into(), (*x1).into(), (*y1).into());

        x0=x1;
        y0=y1;
    }
}

fn decode_relative(rel:u8) -> i16 {
    if (rel & 8) == 8 {
        return 0i16-((rel&7) as i16);
    } else {
        return (rel&7) as i16;
    }
}

fn relative_line<'a, I>(picture:&mut Vec<u8>,priority:&mut Vec<u8>,colour_on:bool,priority_on:bool,colour_pen:u8,priority_pen:u8,volume_iter:&mut Peekable<I>)
where I: Iterator<Item = &'a u8> {

    let mut x0 = *volume_iter.next().unwrap() as i16;
    let mut y0 = *volume_iter.next().unwrap() as i16;

    rasterise_plot(picture, priority, colour_on, priority_on, colour_pen, priority_pen, x0,y0);

    while let Some(b) = volume_iter.peek() {
        if **b >= 0xF0 {
            return;
        }
        let rel = volume_iter.next().unwrap();
        let x1 = x0 + decode_relative(rel>>4);
        let y1 = y0 + decode_relative(rel&0x0F);

        println!("Relative Line : {} {},{} -> {},{}",rel,x0,y0,x1,y1);

        rasterise_line(picture, priority, colour_on, priority_on, colour_pen, priority_pen, x0, y0, x1, y1);

        x0=x1;
        y0=y1;
    }
}


fn fill<'a, I>(picture:&mut Vec<u8>,priority:&mut Vec<u8>,colour_on:bool,priority_on:bool,colour_pen:u8,priority_pen:u8,volume_iter:&mut Peekable<I>)
where I: Iterator<Item = &'a u8> {

    while let Some(b) = volume_iter.peek() {
        if **b >= 0xF0 {
            return;
        }
        let x = volume_iter.next().unwrap();
        let y = volume_iter.next().unwrap();

        println!("Fill at : {},{}",x,y);

        if (colour_on || priority_on) {
            rasterise_fill(picture, priority, colour_on, priority_on, colour_pen, priority_pen, *x, *y);
        }
    }
}

fn plot_pen<'a, I>(picture:&mut Vec<u8>,priority:&mut Vec<u8>,colour_on:bool,priority_on:bool,colour_pen:u8,priority_pen:u8,plot_pen_size:u8,plot_pen_splatter:bool,plot_pen_rectangle:bool,volume_iter:&mut Peekable<I>)
where I: Iterator<Item = &'a u8> {

    if plot_pen_splatter {
        panic!("Splatter pen not implemented");
    }

    while let Some(b) = volume_iter.peek() {
        if **b >= 0xF0 {
            return;
        }
        let x = volume_iter.next().unwrap();
        let y = volume_iter.next().unwrap();

        println!("Plot Pen at : {},{}",x,y);

        rasterise_plot_pen(picture, priority, colour_on, priority_on, colour_pen, priority_pen, plot_pen_size, plot_pen_splatter, plot_pen_rectangle, (*x).into(), (*y).into());
    }

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

fn conv_greyscale(data: &Vec<u8>) -> Vec<u8> {
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

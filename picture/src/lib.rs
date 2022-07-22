use std::{iter::Peekable, collections::VecDeque};

use dir_resource::ResourceDirectoryEntry;
use volume::Volume;

pub const PIC_WIDTH_U8:u8 = 160;
pub const PIC_HEIGHT_U8:u8 = 168;
pub const PIC_WIDTH_USIZE:usize = PIC_WIDTH_U8 as usize;
pub const PIC_HEIGHT_USIZE:usize = PIC_HEIGHT_U8 as usize;

pub struct PictureResource
{
    picture_data:Vec<u8>,
}

impl PictureResource {
    pub fn new(volume:&Volume, entry: &ResourceDirectoryEntry) -> Result<PictureResource, String> {
        let picture_data = volume.fetch_data_slice(entry)?.to_vec();
        Ok(PictureResource { picture_data })
    }

    pub fn render_to(&self,picture:&mut [u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE],priority:&mut [u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE]) -> Result<(), String> {
        *picture = [15u8;(PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE) as usize];
        *priority = [4u8;(PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE) as usize];
        draw_picture(&self.picture_data,picture,priority)?;
        Ok(())
    }
    pub fn render(&self) -> Result<([u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE],[u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE]), String> {
        let mut picture = [15u8;(PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE) as usize];
        let mut priority =[4u8;(PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE) as usize];
        draw_picture(&self.picture_data,&mut picture,&mut priority)?;
        Ok((picture,priority))
    }
}

fn draw_picture(picture_data:&[u8], picture:&mut [u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE], priority:&mut [u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE]) -> Result<(), String> {

    let mut volume_iter = picture_data.iter().peekable();

    let mut colour_pen=15u8;
    let mut priority_pen=4u8;
    let mut colour_on=false;
    let mut priority_on=false;

    let plot_pen_size:u8 = 0;
    let plot_pen_splatter=false;
    let plot_pen_rectangle:bool=true;

    while let Some(b) = volume_iter.next() {
        match b {
            0xF0 => { colour_on=true; colour_pen= *volume_iter.next().unwrap(); },
            0xF1 => { colour_on=false; },
            0xF2 => { priority_on=true; priority_pen= *volume_iter.next().unwrap(); },
            0xF3 => { priority_on=false; },
            0xF4 => { alternate_line(picture,priority,colour_on,priority_on,colour_pen,priority_pen,&mut volume_iter, false); }
            0xF5 => { alternate_line(picture,priority,colour_on,priority_on,colour_pen,priority_pen,&mut volume_iter, true); }
            0xF6 => { absolute_line(picture,priority,colour_on,priority_on,colour_pen,priority_pen,&mut volume_iter); },
            0xF7 => { relative_line(picture,priority,colour_on,priority_on,colour_pen,priority_pen,&mut volume_iter); },
            0xF8 => { fill(picture,priority,colour_on,priority_on,colour_pen,priority_pen,&mut volume_iter); },
            0xFA => { plot_pen(picture,priority,colour_on,priority_on,colour_pen,priority_pen,plot_pen_size,plot_pen_splatter,plot_pen_rectangle,&mut volume_iter); },

            0xFF => break,
            _ => return Err(format!("Unhandled control code {:02X}",b)),
        }
    }

    Ok(())
}

fn rasterise_plot(picture:&mut [u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE],priority:&mut [u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE],colour_on:bool,priority_on:bool,colour_pen:u8,priority_pen:u8, x:i16, y:i16) {

    let width:i16 = PIC_WIDTH_U8.into();
    let height:i16 = PIC_HEIGHT_U8.into();
    if x<0 || x>(width-1) || y<0 || y>(height-1) {
        return;
    }

    let coord = (width*y+x) as usize;
    if colour_on {
        picture[coord]=colour_pen;
    }
    if priority_on {
        priority[coord]=priority_pen;
    }

}

fn rasterise_plot_pen(picture:&mut [u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE],priority:&mut [u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE],colour_on:bool,priority_on:bool,colour_pen:u8,priority_pen:u8,plot_pen_size:u8,plot_pen_splatter:bool,plot_pen_rectangle:bool,x:i16,y:i16) {

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


fn rasterise_line(picture:&mut [u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE],priority:&mut [u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE],colour_on:bool,priority_on:bool,colour_pen:u8,priority_pen:u8, x0:i16, y0:i16, x1:i16, y1:i16) {

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
            error += dy;
            x += sx;
        }
        if e2 <= dx {
            if y == y1 {
                break;
            }
            error += dx;
            y += sy;
        }
    }

}

fn rasterise_fill(picture:&mut [u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE],priority:&mut [u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE],colour_on:bool,priority_on:bool,colour_pen:u8,priority_pen:u8, x:u8, y:u8) {

    let mut queue:VecDeque<(u8,u8)> = VecDeque::new();

    if x>159 || y>199 {
        return;
    }

    queue.push_back((x,y));

    while !queue.is_empty() {

        let (x,y) = queue.pop_front().unwrap();

        let vec_coord = y as usize;
        let vec_coord = vec_coord * (PIC_WIDTH_U8 as usize);
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

        if x<(PIC_WIDTH_U8-1)  { queue.push_back((x+1,y)); }
        if x>0          { queue.push_back((x-1,y)); }
        if y<(PIC_HEIGHT_U8-1) { queue.push_back((x,y+1)); }
        if y>0          { queue.push_back((x,y-1)); }
    }

}

fn alternate_line<'a, I>(picture:&mut [u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE],priority:&mut [u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE],colour_on:bool,priority_on:bool,colour_pen:u8,priority_pen:u8,volume_iter:&mut Peekable<I>, startx:bool)
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
        x= !x;

        rasterise_line(picture, priority, colour_on, priority_on, colour_pen, priority_pen, (*x0).into(), (*y0).into(), (*x1).into(), (*y1).into());

        x0=x1;
        y0=y1;
    }
}


fn absolute_line<'a, I>(picture:&mut [u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE],priority:&mut [u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE],colour_on:bool,priority_on:bool,colour_pen:u8,priority_pen:u8,volume_iter:&mut Peekable<I>)
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

        rasterise_line(picture, priority, colour_on, priority_on, colour_pen, priority_pen, (*x0).into(), (*y0).into(), (*x1).into(), (*y1).into());

        x0=x1;
        y0=y1;
    }
}

fn decode_relative(rel:u8) -> i16 {
    if (rel & 8) == 8 {
        0i16-((rel&7) as i16)
    } else {
        (rel&7) as i16
    }
}

fn relative_line<'a, I>(picture:&mut [u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE],priority:&mut [u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE],colour_on:bool,priority_on:bool,colour_pen:u8,priority_pen:u8,volume_iter:&mut Peekable<I>)
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

        rasterise_line(picture, priority, colour_on, priority_on, colour_pen, priority_pen, x0, y0, x1, y1);

        x0=x1;
        y0=y1;
    }
}


fn fill<'a, I>(picture:&mut [u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE],priority:&mut [u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE],colour_on:bool,priority_on:bool,colour_pen:u8,priority_pen:u8,volume_iter:&mut Peekable<I>)
where I: Iterator<Item = &'a u8> {

    while let Some(b) = volume_iter.peek() {
        if **b >= 0xF0 {
            return;
        }
        let x = volume_iter.next().unwrap();
        let y = volume_iter.next().unwrap();

        if colour_on || priority_on {
            rasterise_fill(picture, priority, colour_on, priority_on, colour_pen, priority_pen, *x, *y);
        }
    }
}

fn plot_pen<'a, I>(picture:&mut [u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE],priority:&mut [u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE],colour_on:bool,priority_on:bool,colour_pen:u8,priority_pen:u8,plot_pen_size:u8,plot_pen_splatter:bool,plot_pen_rectangle:bool,volume_iter:&mut Peekable<I>)
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

        rasterise_plot_pen(picture, priority, colour_on, priority_on, colour_pen, priority_pen, plot_pen_size, plot_pen_splatter, plot_pen_rectangle, (*x).into(), (*y).into());
    }

}

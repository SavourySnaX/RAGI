use std::collections::VecDeque;

use dir_resource::{ResourceDirectoryEntry, ResourceCompression};
use volume::{Volume, VolumeCache};

pub const PIC_WIDTH_U8:u8 = 160;
pub const PIC_HEIGHT_U8:u8 = 168;
pub const PIC_WIDTH_USIZE:usize = PIC_WIDTH_U8 as usize;
pub const PIC_HEIGHT_USIZE:usize = PIC_HEIGHT_U8 as usize;

pub struct PictureResource
{
    picture_data:Vec<u8>,
    compressed:bool,
}

impl PictureResource {
    pub fn new(volume:&Volume, entry: &ResourceDirectoryEntry) -> Result<PictureResource, String> {
        let mut t = VolumeCache::new();
        let data_slice = volume.fetch_data_slice(&mut t,entry)?;
        let picture_data = data_slice.0.to_vec();
        let was_compressed=data_slice.1;
        let compressed = match was_compressed {
            ResourceCompression::Picture => true,
            _ => false,
        };
        Ok(PictureResource { picture_data, compressed })
    }

    pub fn render_onto(&self,picture:&mut [u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE],priority:&mut [u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE]) -> Result<(), String> {
        let mut iter = PictureIterator::new(&self.picture_data,self.compressed);
        draw_picture(&mut iter,picture,priority)?;
        Ok(())
    }
    pub fn render(&self) -> Result<([u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE],[u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE]), String> {
        let mut picture = [15u8;(PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE) as usize];
        let mut priority =[4u8;(PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE) as usize];
        let mut iter = PictureIterator::new(&self.picture_data,self.compressed);
        draw_picture(&mut iter,&mut picture,&mut priority)?;
        Ok((picture,priority))
    }
}

struct PictureIterator<'a> {
    picture_data:&'a [u8],
    position:usize,
    compressed:bool
}

impl PictureIterator<'_> {
    pub fn new(data:&[u8],compressed:bool) -> PictureIterator {
        PictureIterator { picture_data: data, position: 0, compressed }
    }

    fn fetch_nibble(&mut self) -> Option<u8> {
        if self.position/2 >= self.picture_data.len() {
            return None;
        }
        let t = self.picture_data[self.position/2];
        if self.position&1 == 0 {
            Some(t>>4)
        } else {
            Some(t&0xF)
        }
    }
    
    pub fn next_byte(&mut self) -> Option<u8> {
        if let Some(first)=self.fetch_nibble() {
            self.position+=1;
            if let Some(second)=self.fetch_nibble() {
                self.position+=1;
                return Some((first<<4)|second);
            }
        }
        None
    }

    pub fn next_nibble(&mut self) -> Option<u8> {
        if !self.compressed {
            return self.next_byte();
        }
        if let Some(n)=self.fetch_nibble() {
            self.position+=1;
            return Some(n);
        }
        None
    }

    pub fn peek_byte(&mut self) -> Option<u8> {
        if let Some(b) = self.next_byte() {
            self.position-=2;
            return Some(b);
        }
        None
    }

}

fn draw_picture(picture_iter:&mut PictureIterator, picture:&mut [u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE], priority:&mut [u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE]) -> Result<(), String> {

    let mut colour_pen=15u8;
    let mut priority_pen=4u8;
    let mut colour_on=false;
    let mut priority_on=false;

    let mut plot_pen_size:u8 = 0;
    let mut plot_pen_splatter=false;
    let mut plot_pen_rectangle:bool=true;

    while let Some(b) = picture_iter.next_byte() {
        match b {
            0xF0 => { colour_on=true; colour_pen= picture_iter.next_nibble().unwrap(); },
            0xF1 => { colour_on=false; },
            0xF2 => { priority_on=true; priority_pen= picture_iter.next_nibble().unwrap(); },
            0xF3 => { priority_on=false; },
            0xF4 => { alternate_line(picture,priority,colour_on,priority_on,colour_pen,priority_pen,picture_iter, false); }
            0xF5 => { alternate_line(picture,priority,colour_on,priority_on,colour_pen,priority_pen,picture_iter, true); }
            0xF6 => { absolute_line(picture,priority,colour_on,priority_on,colour_pen,priority_pen,picture_iter); },
            0xF7 => { relative_line(picture,priority,colour_on,priority_on,colour_pen,priority_pen,picture_iter); },
            0xF8 => { fill(picture,priority,colour_on,priority_on,colour_pen,priority_pen,picture_iter); },
            0xF9 => { let pstyle = picture_iter.next_byte().unwrap(); plot_pen_size=pstyle&7; plot_pen_rectangle = (pstyle&0x10)==0x10; plot_pen_splatter = (pstyle&0x20)==0x20; }
            0xFA => { plot_pen(picture,priority,colour_on,priority_on,colour_pen,priority_pen,plot_pen_size,plot_pen_splatter,plot_pen_rectangle,picture_iter); },

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

const CIRCLE_SLICE_0:&[u8] = &[1];
const CIRCLE_SLICE_1:&[u8] = &[
    0,0,
    1,1,
    0,0];
const CIRCLE_SLICE_2:&[u8] = &[
    0,1,0,
    1,1,1,
    1,1,1,
    1,1,1,
    0,1,0];
const CIRCLE_SLICE_3:&[u8] = &[
    0,1,1,0,
    0,1,1,0,
    1,1,1,1,
    1,1,1,1,
    1,1,1,1,
    0,1,1,0,
    0,1,1,0];
const CIRCLE_SLICE_4:&[u8] = &[
    0,0,1,0,0,
    0,1,1,1,0,
    1,1,1,1,1,
    1,1,1,1,1,
    1,1,1,1,1,
    1,1,1,1,1,
    1,1,1,1,1,
    0,1,1,1,0,
    0,0,1,0,0];
const CIRCLE_SLICE_5:&[u8] = &[
    0,0,1,1,0,0,
    0,1,1,1,1,0,
    0,1,1,1,1,0,
    0,1,1,1,1,0,
    1,1,1,1,1,1,
    1,1,1,1,1,1,
    1,1,1,1,1,1,
    0,1,1,1,1,0,
    0,1,1,1,1,0,
    0,1,1,1,1,0,
    0,0,1,1,0,0];
const CIRCLE_SLICE_6:&[u8] = &[
    0,0,1,1,1,0,0,
    0,1,1,1,1,1,0,
    0,1,1,1,1,1,0,
    0,1,1,1,1,1,0,
    1,1,1,1,1,1,1,
    1,1,1,1,1,1,1,
    1,1,1,1,1,1,1,
    1,1,1,1,1,1,1,
    1,1,1,1,1,1,1,
    0,1,1,1,1,1,0,
    0,1,1,1,1,1,0,
    0,1,1,1,1,1,0,
    0,0,1,1,1,0,0];
const CIRCLE_SLICE_7:&[u8] = &[
    0,0,0,1,1,0,0,0,
    0,0,1,1,1,1,0,0,
    0,1,1,1,1,1,1,0,
    0,1,1,1,1,1,1,0,
    0,1,1,1,1,1,1,0,
    1,1,1,1,1,1,1,1,
    1,1,1,1,1,1,1,1,
    1,1,1,1,1,1,1,1,
    1,1,1,1,1,1,1,1,
    1,1,1,1,1,1,1,1,
    0,1,1,1,1,1,1,0,
    0,1,1,1,1,1,1,0,
    0,1,1,1,1,1,1,0,
    0,0,1,1,1,1,0,0,
    0,0,0,1,1,0,0,0];

const CIRCLE_SLICE_TABLE:&[&[u8]] = &[CIRCLE_SLICE_0,CIRCLE_SLICE_1,CIRCLE_SLICE_2,CIRCLE_SLICE_3,CIRCLE_SLICE_4,CIRCLE_SLICE_5,CIRCLE_SLICE_6,CIRCLE_SLICE_7];

const TEXTURE_DATA_SLICE:&[u8] = &[
    0x20, 0x94, 0x02, 0x24, 0x90, 0x82, 0xa4, 0xa2,
    0x82, 0x09, 0x0a, 0x22, 0x12, 0x10, 0x42, 0x14,
    0x91, 0x4a, 0x91, 0x11, 0x08, 0x12, 0x25, 0x10,
    0x22, 0xa8, 0x14, 0x24, 0x00, 0x50, 0x24, 0x04];

const TEXTURE_DATA_BIT_OFFSET:&[u8] = &[
    0x00, 0x18, 0x30, 0xc4, 0xdc, 0x65, 0xeb, 0x48,
    0x60, 0xbd, 0x89, 0x04, 0x0a, 0xf4, 0x7d, 0x6d,
    0x85, 0xb0, 0x8e, 0x95, 0x1f, 0x22, 0x0d, 0xdf,
    0x2a, 0x78, 0xd5, 0x73, 0x1c, 0xb4, 0x40, 0xa1,
    0xb9, 0x3c, 0xca, 0x58, 0x92, 0x34, 0xcc, 0xce,
    0xd7, 0x42, 0x90, 0x0f, 0x8b, 0x7f, 0x32, 0xed,
    0x5c, 0x9d, 0xc8, 0x99, 0xad, 0x4e, 0x56, 0xa6,
    0xf7, 0x68, 0xb7, 0x25, 0x82, 0x37, 0x3a, 0x51,
    0x69, 0x26, 0x38, 0x52, 0x9e, 0x9a, 0x4f, 0xa7,
    0x43, 0x10, 0x80, 0xee, 0x3d, 0x59, 0x35, 0xcf,
    0x79, 0x74, 0xb5, 0xa2, 0xb1, 0x96, 0x23, 0xe0,
    0xbe, 0x05, 0xf5, 0x6e, 0x19, 0xc5, 0x66, 0x49,
    0xf0, 0xd1, 0x54, 0xa9, 0x70, 0x4b, 0xa4, 0xe2,
    0xe6, 0xe5, 0xab, 0xe4, 0xd2, 0xaa, 0x4c, 0xe3,
    0x06, 0x6f, 0xc6, 0x4a, 0x75, 0xa3, 0x97, 0xe1];

fn is_pixel_texture_on(bit_pos:u8) -> (u8,bool) {
    let byte = bit_pos/8;
    let bit = 0x80 >> (bit_pos&7);
    let on = (TEXTURE_DATA_SLICE[byte as usize] & bit) == bit;
    let ret = bit_pos+1;
    if ret == 255 {
        return (0,on);
    }
    (ret,on)
}

fn rasterise_plot_pen(picture:&mut [u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE],priority:&mut [u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE],colour_on:bool,priority_on:bool,colour_pen:u8,priority_pen:u8,plot_pen_size:u8,plot_pen_splatter:bool,plot_pen_rectangle:bool,t:u8,x:i16,y:i16) {

    let w = (plot_pen_size as i16)+1;
    let h = (plot_pen_size as i16)*2+1;
    let sx = x-(w/2);
    let sy = y-h/2;

    let mut bit_pos = TEXTURE_DATA_BIT_OFFSET[(t>>1) as usize];

    let mut on:bool;
    if !plot_pen_rectangle {

        let circle_pixel = CIRCLE_SLICE_TABLE[plot_pen_size as usize];
        let mut iter=0;
        for y in sy..sy+h {
            for x in sx..sx+w {
                // check circle from rect shape
                if circle_pixel[iter]==1 {
                    (bit_pos,on) = is_pixel_texture_on(bit_pos);
                    if on || (!plot_pen_splatter) {
                        rasterise_plot(picture, priority, colour_on, priority_on, colour_pen, priority_pen, x, y);
                    }
                }
                iter+=1;
            }
        }
    } else {
        // Rectangle renderer 
        for y in sy..sy+h {
            for x in sx..sx+w {
                (bit_pos,on) = is_pixel_texture_on(bit_pos);
                if on || (!plot_pen_splatter) {
                    rasterise_plot(picture, priority, colour_on, priority_on, colour_pen, priority_pen, x, y);
                }
            }
        }
    }

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

    if colour_on && colour_pen==15 {
        return;
    }
    if priority_on && !colour_on && priority_pen==4 {
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

fn alternate_line(picture:&mut [u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE],priority:&mut [u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE],colour_on:bool,priority_on:bool,colour_pen:u8,priority_pen:u8,picture_iter:&mut PictureIterator, startx:bool) {

    let mut x0 = picture_iter.next_byte().unwrap();
    let mut y0 = picture_iter.next_byte().unwrap();

    let mut x=startx;
    let mut x1= x0;
    let mut y1= y0;

    rasterise_plot(picture, priority, colour_on, priority_on, colour_pen, priority_pen, x0.into(), y0.into());

    while let Some(b) = picture_iter.peek_byte() {
        if b >= 0xF0 {
            return;
        }

        let n = picture_iter.next_byte().unwrap();

        if x {
            x1 = n;
        } else {
            y1 = n;
        }
        x= !x;

        rasterise_line(picture, priority, colour_on, priority_on, colour_pen, priority_pen, x0.into(), y0.into(), x1.into(), y1.into());

        x0=x1;
        y0=y1;
    }
}


fn absolute_line(picture:&mut [u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE],priority:&mut [u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE],colour_on:bool,priority_on:bool,colour_pen:u8,priority_pen:u8,picture_iter:&mut PictureIterator) {

    let mut x0 = picture_iter.next_byte().unwrap();
    let mut y0 = picture_iter.next_byte().unwrap();

    rasterise_plot(picture, priority, colour_on, priority_on, colour_pen, priority_pen, x0.into(), y0.into());

    while let Some(b) = picture_iter.peek_byte() {
        if b >= 0xF0 {
            return;
        }
        let x1 = picture_iter.next_byte().unwrap();
        let y1 = picture_iter.next_byte().unwrap();

        rasterise_line(picture, priority, colour_on, priority_on, colour_pen, priority_pen, x0.into(), y0.into(),x1.into(),y1.into());

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

fn relative_line(picture:&mut [u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE],priority:&mut [u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE],colour_on:bool,priority_on:bool,colour_pen:u8,priority_pen:u8,picture_iter:&mut PictureIterator) {

    let mut x0 = picture_iter.next_byte().unwrap() as i16;
    let mut y0 = picture_iter.next_byte().unwrap() as i16;

    rasterise_plot(picture, priority, colour_on, priority_on, colour_pen, priority_pen, x0,y0);

    while let Some(b) = picture_iter.peek_byte() {
        if b >= 0xF0 {
            return;
        }
        let rel = picture_iter.next_byte().unwrap();
        let x1 = x0 + decode_relative(rel>>4);
        let y1 = y0 + decode_relative(rel&0x0F);

        rasterise_line(picture, priority, colour_on, priority_on, colour_pen, priority_pen, x0, y0, x1, y1);

        x0=x1;
        y0=y1;
    }
}


fn fill(picture:&mut [u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE],priority:&mut [u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE],colour_on:bool,priority_on:bool,colour_pen:u8,priority_pen:u8,picture_iter:&mut PictureIterator) {

    while let Some(b) = picture_iter.peek_byte() {
        if b >= 0xF0 {
            return;
        }
        let x = picture_iter.next_byte().unwrap();
        let y = picture_iter.next_byte().unwrap();

        if colour_on || priority_on {
            rasterise_fill(picture, priority, colour_on, priority_on, colour_pen, priority_pen, x, y);
        }
    }
}

fn plot_pen(picture:&mut [u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE],priority:&mut [u8;PIC_WIDTH_USIZE*PIC_HEIGHT_USIZE],colour_on:bool,priority_on:bool,colour_pen:u8,priority_pen:u8,plot_pen_size:u8,plot_pen_splatter:bool,plot_pen_rectangle:bool,picture_iter:&mut PictureIterator) {

    while let Some(b) = picture_iter.peek_byte() {
        if b >= 0xF0 {
            return;
        }
        let mut t = 0u8;
        if plot_pen_splatter {
            t = picture_iter.next_byte().unwrap();
        }
        let x = picture_iter.next_byte().unwrap();
        let y = picture_iter.next_byte().unwrap();

        rasterise_plot_pen(picture, priority, colour_on, priority_on, colour_pen, priority_pen, plot_pen_size, plot_pen_splatter, plot_pen_rectangle, t, x.into(), y.into());
    }

}

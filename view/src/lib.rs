use dir_resource::ResourceDirectoryEntry;
use volume::{Volume, VolumeCache};

pub struct ViewCel {
    width:u8,
    height:u8,
    flags:u8,
    data:Vec<u8>,
}

pub struct ViewLoop {
    cels:Vec<ViewCel>
}
pub struct ViewResource {
    description:String,
    loops:Vec<ViewLoop>,
}

impl ViewCel {
    pub fn get_transparent_colour(&self) -> u8 {
        self.flags&0xF
    }

    pub fn is_mirror(&self,cloop:u8) -> bool {
        if self.flags&0x80 == 0x80 {
            if ((self.flags&0x70)>>5) != cloop {
                return true;
            }
        }
        false
    }

    pub fn get_width(&self) -> u8 {
        self.width
    }

    pub fn get_height(&self) -> u8 {
        self.height
    }

    pub fn get_data(&self) -> &Vec<u8> {
        &self.data
    }
}

impl ViewLoop {
    pub fn get_cels(&self) -> &Vec<ViewCel> {
        &self.cels
    }
}

impl ViewResource {
    pub fn new(volume:&Volume, entry: &ResourceDirectoryEntry) -> Result<ViewResource, String> {
        let mut t=VolumeCache::new();
        let slice = volume.fetch_data_slice(&mut t,entry)?;
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

        let mut description = String::new();
        if description_position!=0 {
            let slice = &slice[description_position..];
            let mut iter = slice.iter();

            loop {
                let b = iter.next().unwrap();
                if *b==0 {
                    break;
                }
                description = description + &String::from((*b) as char);
            }
        }

        let mut loops:Vec<ViewLoop>= Vec::new();
        loops.reserve(loop_positions.len());
        for l in loop_positions {

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

            let mut cels:Vec<ViewCel>=Vec::new();
            cels.reserve(cell_positions.len());
            for c in cell_positions {

                let slice = &slice[c..];

                let mut iter = slice.iter();

                let width = iter.next().unwrap();
                let height = iter.next().unwrap();
                let flags = iter.next().unwrap();

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
                cels.push(ViewCel { width: *width, height: *height, flags: *flags, data: image });
            }
            loops.push(ViewLoop { cels });
        }

        Ok(ViewResource {description, loops})
    }

    pub fn get_description(&self) ->&String {
        &self.description
    }

    pub fn get_loops(&self) -> &Vec<ViewLoop> {
        &self.loops
    }

}
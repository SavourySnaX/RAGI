use std::fs;

use helpers::*;
use dir_resource::{ResourceDirectory, ResourceDirectoryEntry, Root, ResourceType};
use picture::*;
use volume::{Volume, VolumeCache};


fn main() {

    //let root = Root::new("../images/Leisure Suit Larry in the Land of the Lounge Lizards (1987)(Sierra On-Line, Inc.) [Adventure]/");
    //let root=Root::new("../images/King's Quest v1.0U (1986)(Sierra On-Line, Inc.) [Adventure][!]/");
    //let root = Root::new("../images/Space Quest- The Sarien Encounter v1.0X (1986)(Sierra On-Line, Inc.) [Adventure]/","2.089");
    let root = Root::new("../images/Gold Rush! v2.01 (1988)(Sierra On-Line, Inc.) [Adventure]/","3.002.149");
    let dir = ResourceDirectory::real_new(&root, ResourceType::Pictures).unwrap();

    for (index,entry) in dir.into_iter().enumerate() {
        if !entry.empty() {
            println!("{} : V{} P{}",index,entry.volume,entry.position);
            dump_picture_resource(&root,&entry, index);
        }
    }

}

fn dump_picture_resource(root:&Root,entry:&ResourceDirectoryEntry, index:usize) {

    let bytes = root.fetch_volume(entry);

    let mut t = VolumeCache::new();
    let volume = Volume::new(bytes.into_iter()).unwrap();

    let data_slice = volume.fetch_data_slice(&mut t,entry).unwrap();
    let volume_iter = data_slice.0.iter();

    let data:Vec<u8> = volume_iter.cloned().collect();
    fs::write(format!("../{}-binary.bin",index).as_str(),data).unwrap();

    let picture_resource = match PictureResource::new(&volume,entry) {
        Ok(b) => b,
        Err(s) => panic!("Failed due to : {}", s),
    };
    let (picture,priority) = picture_resource.render().unwrap();
    
    let doubled_width = double_pic_width(&picture);

    let rgba = conv_rgba(&doubled_width);

    let width:u32 = PIC_WIDTH_U8.into();
    let height:u32 = PIC_HEIGHT_U8.into();

    dump_png(format!("../{}-picture.png",index).as_str(),width*2,height,&rgba);

    let doubled_width = double_pic_width(&priority);

    let rgba = conv_greyscale(&doubled_width);

    dump_png(format!("../{}-priority.png",index).as_str(),width*2,height,&rgba);

}

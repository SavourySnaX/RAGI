use std::fs;

use helpers::*;
use dir_resource::{ResourceDirectory, ResourceDirectoryEntry};
use picture::*;
use volume::Volume;


fn main() {

    //let root = Root::new("../images/Leisure Suit Larry in the Land of the Lounge Lizards (1987)(Sierra On-Line, Inc.) [Adventure]/");
    let root = Root::new("../images/Space Quest- The Sarien Encounter v1.0X (1986)(Sierra On-Line, Inc.) [Adventure]/");

    let bytes = root.read_data_or_default("PICDIR");

    let dir = ResourceDirectory::new(bytes.into_iter()).unwrap();

    for (index,entry) in dir.into_iter().enumerate() {
        if !entry.empty() {
            println!("{} : V{} P{}",index,entry.volume,entry.position);
            dump_picture_resource(&root,&entry, index);
        }
    }

}

fn dump_picture_resource(root:&Root,entry:&ResourceDirectoryEntry, index:usize) {

    let bytes = root.read_data_or_default(format!("VOL.{}", entry.volume).as_str());

    let volume = Volume::new(bytes.into_iter()).unwrap();

    let picture_resource = match PictureResource::new(&volume,entry) {
        Ok(b) => b,
        Err(s) => panic!("Failed due to : {}", s),
    };
    let (picture,priority) = picture_resource.render().unwrap();
    
    let doubled_width = double_width(&picture);

    let rgba = conv_rgba(&doubled_width);

    let width:u32 = WIDTH.into();
    let height:u32 = HEIGHT.into();

    dump_png(format!("../{}-picture.png",index).as_str(),width*2,height,&rgba);

    let doubled_width = double_width(&priority);

    let rgba = conv_greyscale(&doubled_width);

    dump_png(format!("../{}-priority.png",index).as_str(),width*2,height,&rgba);

    let volume_iter = volume.fetch_data_slice(entry).unwrap().iter();

    let data:Vec<u8> = volume_iter.cloned().collect();
    fs::write(format!("../{}-binary.bin",index).as_str(),data).unwrap();
}

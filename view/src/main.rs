
use std::fs;

use dir_resource::{ResourceDirectory, ResourceDirectoryEntry, Root, ResourceType};
use helpers::*;
use view::ViewResource;
use volume::{Volume, VolumeCache};

fn main() {

    //let root = Root::new("../images/Leisure Suit Larry in the Land of the Lounge Lizards (1987)(Sierra On-Line, Inc.) [Adventure]/");
    //let root = Root::new("../images/Space Quest- The Sarien Encounter v1.0X (1986)(Sierra On-Line, Inc.) [Adventure]/","2.089");
    let root = Root::new("../images/Gold Rush! v2.01 (1988)(Sierra On-Line, Inc.) [Adventure]/","3.002.149");
    let dir = ResourceDirectory::real_new(&root, ResourceType::Views).unwrap();

    for (index,entry) in dir.into_iter().enumerate() {
        if !entry.empty() {
            println!("{} : V{} P{}",index,entry.volume,entry.position);
            dump_view_resource(&root,&entry, index);
        }
    }

}

fn dump_view_resource(root:&Root,entry:&ResourceDirectoryEntry, index:usize) {

    let bytes = root.fetch_volume(entry);

    let mut t = VolumeCache::new();
    let volume = Volume::new(bytes.into_iter()).unwrap();

    let data = volume.fetch_data_slice(&mut t,entry).unwrap();

    fs::write(format!("../{}-binary.bin",index).as_str(),data).unwrap();

    let view = match ViewResource::new(&volume, entry) {
        Ok(b) => b,
        Err(s) => panic!("Failed due to : {}", s),
    };

    println!("{}-Description : {}",index, view.get_description());
    for (l_index,l) in view.get_loops().iter().enumerate() {
        for (c_index,c) in l.get_cels().iter().enumerate() {
            let doubled_width = double_width(c.get_data());

            let rgba = conv_rgba_transparent(&doubled_width, c.get_transparent_colour());

            dump_png(format!("../{}-cell-{}-{}.png",index, l_index, c_index).as_str(),(c.get_width() as u32)*2,c.get_height() as u32,&rgba);
        }
    }



}

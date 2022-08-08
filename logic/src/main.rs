
use std::{fs};

use dir_resource::{ResourceDirectory, ResourceDirectoryEntry, Root, ResourceType};
use logic::LogicResource;
use objects::Objects;
use volume::{Volume, VolumeCache};
use words::Words;

fn main() {

    //let root=Root::new("../images/King's Quest v1.0U (1986)(Sierra On-Line, Inc.) [Adventure][!]/","2.272");
    //let root = Root::new("../images/Leisure Suit Larry in the Land of the Lounge Lizards (1987)(Sierra On-Line, Inc.) [Adventure]/", "2.440");
    //let root = Root::new("../images/Space Quest- The Sarien Encounter v1.0X (1986)(Sierra On-Line, Inc.) [Adventure]/","2.089");
    //let root = Root::new("../images/King's Quest II- Romancing the Throne v2.1 (1987)(Sierra On-Line, Inc.) [Adventure]/","2.411");
    let root = Root::new("../images/Gold Rush! v2.01 (1988)(Sierra On-Line, Inc.) [Adventure]/","3.002.149");
    //let root = Root::new("../images/Black Cauldron, The v2.10 (1988)(Sierra On-Line, Inc.) [Adventure]/","3.002.098");
    //let root = Root::new("../images/GROZA/");

    let bytes = root.read_data_or_default("OBJECT");

    let items = match Objects::new(&bytes) {
        Ok(a) => a,
        Err(_) => panic!("!"),//Objects::blank(),
    };
    
    let bytes = root.read_data_or_default("WORDS.TOK");

    let words = match Words::new(bytes.into_iter()) {
        Ok(a) => a,
        Err(_) => panic!("!"),//Words::blank(),
    };

    let dir = ResourceDirectory::real_new(&root, ResourceType::Logic).unwrap();

    for (index,entry) in dir.into_iter().enumerate() {
        if !entry.empty() {
            println!("{} : V{} P{}",index,entry.volume,entry.position);
            dump_logic_resource(&root,&entry,index,&items,&words);
        }
    }

}

fn dump_logic_resource(root:&Root,entry:&ResourceDirectoryEntry,index:usize,items:&Objects,words:&Words) {

    let bytes = root.fetch_volume(entry);

    let mut t = VolumeCache::new();
    let volume = Volume::new(bytes.into_iter()).unwrap();

    let data_slice = volume.fetch_data_slice(&mut t,entry).unwrap();
    let data = data_slice.0;

    fs::write(format!("../{}-binary.bin",index).as_str(),data).unwrap();

    let logic_resource = LogicResource::new(&volume,entry,root.version()).unwrap();

    logic_resource.disassemble(items,words);
}

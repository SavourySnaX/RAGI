
use std::{fs};
use std::{path::Path};

use dir_resource::{ResourceDirectory, ResourceDirectoryEntry};
use logic::LogicResource;
use objects::Objects;
use volume::{Volume, VolumeCache};
use words::Words;

struct Root<'a> {
    base_path:&'a Path,
}

impl<'a> Root<'_> {
    pub fn new(base_path:&'a str) -> Root {
        Root {base_path:Path::new(base_path)}
    }
}

fn main() {

    //let root=Root::new("../images/King's Quest v1.0U (1986)(Sierra On-Line, Inc.) [Adventure][!]/"); let version = "2.272";
    let root = Root::new("../images/Leisure Suit Larry in the Land of the Lounge Lizards (1987)(Sierra On-Line, Inc.) [Adventure]/"); let version="2.440";
    //let root = Root::new("../images/Space Quest- The Sarien Encounter v1.0X (1986)(Sierra On-Line, Inc.) [Adventure]/"); let version="2.089";
    //let root = Root::new("../images/GROZA/");

    let bytes = fs::read(root.base_path.join("OBJECT").into_os_string()).unwrap_or_default();

    let items = match Objects::new(&bytes) {
        Ok(a) => a,
        Err(_) => panic!("!"),//Objects::blank(),
    };
    
    let bytes = fs::read(root.base_path.join("WORDS.TOK").into_os_string()).unwrap_or_default();

    let words = match Words::new(bytes.into_iter()) {
        Ok(a) => a,
        Err(_) => panic!("!"),//Words::blank(),
    };

    let bytes = fs::read(root.base_path.join("LOGDIR").into_os_string()).unwrap_or_default();

    let dir = ResourceDirectory::new(bytes).unwrap();

    for (index,entry) in dir.into_iter().enumerate() {
        if !entry.empty() {
            println!("{} : V{} P{}",index,entry.volume,entry.position);
            dump_logic_resource(&root,&entry,index,&items,&words,version);
        }
    }

}

fn dump_logic_resource(root:&Root,entry:&ResourceDirectoryEntry,index:usize,items:&Objects,words:&Words,version:&str) {

    let bytes = fs::read(root.base_path.join(format!("VOL.{}", entry.volume)).into_os_string()).unwrap_or_default();

    let mut t = VolumeCache::new();
    let volume = Volume::new(bytes.into_iter()).unwrap();

    let data = volume.fetch_data_slice(&mut t,entry).unwrap();

    fs::write(format!("../{}-binary.bin",index).as_str(),data).unwrap();

    let logic_resource = LogicResource::new(&volume,entry,version).unwrap();

    logic_resource.disassemble(items,words);
}

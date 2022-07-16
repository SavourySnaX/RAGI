
use std::{fs};
use std::{path::Path};

use dir_resource::{ResourceDirectory, ResourceDirectoryEntry};
use logic::{LogicResource, LogicMessages};
use objects::Objects;
use volume::Volume;
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

    //let root = Root::new("../images/Leisure Suit Larry in the Land of the Lounge Lizards (1987)(Sierra On-Line, Inc.) [Adventure]/");
    let root = Root::new("../images/Space Quest- The Sarien Encounter v1.0X (1986)(Sierra On-Line, Inc.) [Adventure]/");

    let bytes = fs::read(root.base_path.join("OBJECT").into_os_string()).unwrap_or_default();

    let items = Objects::new(&bytes).unwrap();
    
    let bytes = fs::read(root.base_path.join("WORDS.TOK").into_os_string()).unwrap_or_default();

    let words = Words::new(bytes.into_iter()).unwrap();

    let bytes = fs::read(root.base_path.join("LOGDIR").into_os_string()).unwrap_or_default();

    let dir = ResourceDirectory::new(bytes.into_iter()).unwrap();

    for (index,entry) in dir.into_iter().enumerate() {
        if !entry.empty() {
            println!("{} : V{} P{}",index,entry.volume,entry.position);
            dump_logic_resource(&root,&entry,index,&items,&words);
        }
    }

}

fn dump_logic_resource(root:&Root,entry:&ResourceDirectoryEntry,index:usize,items:&Objects,words:&Words) {

    let bytes = fs::read(root.base_path.join(format!("VOL.{}", entry.volume)).into_os_string()).unwrap_or_default();

    let volume = Volume::new(bytes.into_iter()).unwrap();

    let data = volume.fetch_data_slice(entry).unwrap();

    fs::write(format!("../{}-binary.bin",index).as_str(),data).unwrap();

    let logic_resource = LogicResource::new(&volume,entry).unwrap();

    logic_resource.disassemble(items,words);
}

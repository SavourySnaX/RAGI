use std::fs;

use dir_resource::ResourceDirectory;

fn main() {

    let bytes = fs::read("../images/Space Quest- The Sarien Encounter v1.0X (1986)(Sierra On-Line, Inc.) [Adventure]/LOGDIR").unwrap_or_default();

    let dir = ResourceDirectory::new(bytes.into_iter());

    for (i,iter) in dir.unwrap().into_iter().enumerate() {
        if iter.empty() {
            println!("Resource Entry {:02X} | EMPTY",i);
        } else {
            println!("Resource Entry {:02X} | Volume {:01X} , Position {:05X}", i, iter.volume, iter.position);
        }
    }

}

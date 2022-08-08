use dir_resource::{ResourceDirectory, Root};

fn main() {
    let root = Root::new("../images/Space Quest- The Sarien Encounter v1.0X (1986)(Sierra On-Line, Inc.) [Adventure]/","2.089");
    //let root = Root::new("../images/Gold Rush! v2.01 (1988)(Sierra On-Line, Inc.) [Adventure]/","3.002.149");

    let dir = ResourceDirectory::new(&root,dir_resource::ResourceType::Logic);

    for (i,iter) in dir.unwrap().into_iter().enumerate() {
        if iter.empty() {
            println!("Resource Entry {:02X} | EMPTY",i);
        } else {
            println!("Resource Entry {:02X} | Volume {:01X} , Position {:05X}", i, iter.volume, iter.position);
        }
    }

}

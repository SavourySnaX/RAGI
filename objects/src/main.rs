use std::fs;

use objects::Objects;

fn main() {

    let bytes = fs::read("../images/Gold Rush! v2.01 (1988)(Sierra On-Line, Inc.) [Adventure]/OBJECT").unwrap_or_default();

    let objects = Objects::new(&bytes).unwrap();

    println!("Max Objects : {}",objects.max_objects);
    for (index,object) in objects.objects.iter().enumerate() {
        println!("{}: \"{}\" - {}",index,object.name,object.start_room);
    }
}
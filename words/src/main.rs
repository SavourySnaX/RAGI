use std::fs;

use words::Words;

fn main() {

    let bytes = fs::read("../images/Space Quest- The Sarien Encounter v1.0X (1986)(Sierra On-Line, Inc.) [Adventure]/WORDS.TOK").unwrap_or_default();

    let words = Words::new(bytes.into_iter()).unwrap();

    for word in words {
        println!("{} - {}",word.0,word.1);
    }


}

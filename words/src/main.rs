use std::fs;

use words::Words;

fn main() {

    let bytes=fs::read("../images/Leisure Suit Larry in the Land of the Lounge Lizards (1987)(Sierra On-Line, Inc.) [Adventure]/WORDS.TOK").unwrap_or_default();

    let words = Words::new(bytes.into_iter()).unwrap();

    for word in words {
        println!("{} - {}",word.0,word.1);
    }
}

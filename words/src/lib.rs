use std::{collections::HashMap, ops::Index};

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn construct_ok0() {
        assert!(Words::new([0u8;52].into_iter()).is_ok());
    }

    #[test]
    fn construct_ok_a() {
        let mut test_data:Vec<u8> = Vec::from([0u8;52]);
        test_data.extend_from_slice(&[0u8,(b'a'^0x7F)|0x80,0x12,0x34]);
        let words = Words::new(test_data.into_iter()).unwrap();
        assert_eq!(words[String::from("a")],0x1234);
    }

    #[test]
    fn construct_fail_empty() {
        assert!(Words::new([0u8;52].into_iter()).is_err());
    }
}

pub struct Words {
    pub words : HashMap<String,u16>,
}

impl Index<String> for Words {
    type Output = u16;

    fn index(&self, index: String) -> &Self::Output {
        self.words.index(&index)
    }
}

impl Words {
    pub fn new(bytes: impl Iterator<Item = u8>) -> Result<Words,&'static str> {

        let mut words: HashMap<String,u16> = HashMap::new();

        // We don't need the starting letter jump tables, so just skip them
        let mut bytes=bytes.skip(52);

        let mut last_word = String::new();

        while let Some(b) = bytes.next() {

            // First byte = num chars from previous word
            last_word=last_word.chars().into_iter().take(b as usize).collect();

            while let Some(b) = bytes.next() {
                let b = b ^ 0x7F;
                let is_last = b&0x80==0x80;
                let b=b&0x7F;
                last_word.push(b as char);
                if is_last {
                    break;
                }
            }

            if !last_word.is_empty() {
                let word_num:u16 = bytes.next().unwrap().into();
                let word_num = word_num<<8;
                let t:u16 = bytes.next().unwrap().into();
                let word_num = word_num + t;

                words.insert(last_word.clone(), word_num);
            }
        }
        if words.len()==0 {
            return Err("There should be at least 1 word!");
        }
        Ok(Words {words})
    }
}
use std::{ops::Index};

#[cfg(test)]
mod tests {
    use crate::ResourceDirectory;

    #[test]
    fn construct_ok0() {
        assert!(ResourceDirectory::new([0u8;0].into_iter()).is_ok());
    }
    #[test]
    fn construct_ok3() {
        assert!(ResourceDirectory::new([0u8;3].into_iter()).is_ok());
    }
    #[test]
    fn construct_ok9() {
        assert!(ResourceDirectory::new([0u8;9].into_iter()).is_ok());
    }
    #[test]
    fn construct_fail1() {
        assert!(ResourceDirectory::new([0u8;1].into_iter()).is_err());
    }
    #[test]
    fn construct_fail2() {
        assert!(ResourceDirectory::new([0u8;2].into_iter()).is_err());
    }
    #[test]
    fn construct_fail4() {
        assert!(ResourceDirectory::new([0u8;4].into_iter()).is_err());
    }

    #[test]
    fn get_present() {
        let d = ResourceDirectory::new([0u8;3].into_iter()).unwrap();
        assert!(d.get(0).is_some())
    }

    #[test]
    fn get_out_of_range() {
        let d = ResourceDirectory::new([0u8;3].into_iter()).unwrap();
        assert!(d.get(1).is_none())
    }

    #[test]
    fn empty_check() {
        let d = ResourceDirectory::new([255u8;3].into_iter()).unwrap();
        assert!(d.get(0).unwrap().empty());
    }

    #[test]
    fn not_empty_check() {
        let d = ResourceDirectory::new([0u8;3].into_iter()).unwrap();
        assert!(!d.get(0).unwrap().empty());
    }

}

/// Represents an entry in a Directory Resource in AGI
pub struct ResourceDirectoryEntry {
    pub volume:u8,
    pub position:u32,
}

/// Represents a Directory Resource in AGI (e.g. PICDIR)
pub struct ResourceDirectory {
    entries:Vec<ResourceDirectoryEntry>,
}

impl IntoIterator for ResourceDirectory {
    type Item = ResourceDirectoryEntry;
    type IntoIter = ::std::vec::IntoIter<ResourceDirectoryEntry>;
    
    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

impl<'a> IntoIterator for &'a ResourceDirectory {
    type Item = &'a ResourceDirectoryEntry;
    type IntoIter = ::std::slice::Iter<'a, ResourceDirectoryEntry>;
    
    fn into_iter(self) -> Self::IntoIter {
        self.entries.iter()
    }
}

impl ResourceDirectoryEntry {
    pub fn empty(&self) -> bool {
        self.volume==0xF
    }
}

impl Index<usize> for ResourceDirectory {
    type Output = ResourceDirectoryEntry;

    fn index(&self, idx:usize) -> &Self::Output {
        self.entries.index(idx)
    }
}

impl ResourceDirectory {
    pub fn new(mut bytes: impl Iterator<Item = u8>) -> Result<ResourceDirectory, &'static str> {

        let mut entries = Vec::new();

        while let Some(b) = bytes.next() {
            let volume = b>>4;
            let position:u32 = (b&0xF).into();

            if let Some(b) = bytes.next() {
                let t:u32 = b.into();
                let position:u32 = (position<<8) + t;

                if let Some(b) = bytes.next() {
                    let t:u32 = b.into();
                    let position:u32 = (position<<8) + t;
                    entries.push(ResourceDirectoryEntry { volume, position});
                } else {
                    return Err("Input bytes are not made up of triples (size % 3 != 0)");
                }
            } else {
                return Err("Input bytes are not made up of triples (size % 3 != 0)");
            }
        }
        return Ok(ResourceDirectory{entries});
    }

    pub fn get<'a>(&'a self,index: usize) -> Option<&'a ResourceDirectoryEntry> {
        return self.entries.get(index);
    }

}
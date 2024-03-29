use std::{ops::Index, path::Path, fs, cmp::Ordering};

#[cfg(test)]
mod tests {
    use crate::ResourceDirectory;
/*
    #[test]
    fn construct_ok0() {
        assert!(ResourceDirectory::new(vec![0u8;0]).is_ok());
    }
    #[test]
    fn construct_ok3() {
        assert!(ResourceDirectory::new(vec![0u8;3]).is_ok());
    }
    #[test]
    fn construct_ok9() {
        assert!(ResourceDirectory::new(vec![0u8;9]).is_ok());
    }
    #[test]
    fn construct_fail1() {
        assert!(ResourceDirectory::new(vec![0u8;1]).is_err());
    }
    #[test]
    fn construct_fail2() {
        assert!(ResourceDirectory::new(vec![0u8;2]).is_err());
    }
    #[test]
    fn construct_fail4() {
        assert!(ResourceDirectory::new(vec![0u8;4]).is_err());
    }

    #[test]
    fn get_present() {
        let d = ResourceDirectory::new(vec![0u8;3]).unwrap();
        assert!(d.get(0).is_some())
    }

    #[test]
    fn get_out_of_range() {
        let d = ResourceDirectory::new(vec![0u8;3]).unwrap();
        assert!(d.get(1).is_none())
    }

    #[test]
    fn empty_check() {
        let d = ResourceDirectory::new(vec![255u8;3]).unwrap();
        assert!(d.get(0).unwrap().empty());
    }

    #[test]
    fn not_empty_check() {
        let d = ResourceDirectory::new(vec![0u8;3]).unwrap();
        assert!(!d.get(0).unwrap().empty());
    }
*/
}


#[derive(Eq,Clone,Copy)]
pub struct ResourcesVersion {
    comparing:u64,
}

impl std::fmt::Display for ResourcesVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}.{}.{}",self.comparing>>32,(self.comparing>>16)&0xFFFF,self.comparing&0xFFFF)
    }
}

pub struct Root<'a> {
    base_path:&'a Path,
    version:ResourcesVersion,
}

impl ResourcesVersion {
    pub fn new(str:&str) -> ResourcesVersion {
        let mut parts = str.split('.');
        let mut major=0;
        let mut minor=0;
        let mut patch=0;
        if let Some(smajor) = parts.next() {
            major = smajor.parse::<u8>().unwrap_or_default();
            if let Some(sminor) = parts.next() {
                minor = sminor.parse::<u16>().unwrap_or_default();
                if let Some(spatch) = parts.next() {
                    patch = spatch.parse::<u16>().unwrap_or_default();
                }
            }
        }
        let comparing = (major as u64)<<32;
        let comparing = comparing + ((minor as u64)<<16);
        let comparing = comparing + (patch as u64);
        ResourcesVersion { comparing }
    }
}

impl PartialOrd for ResourcesVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for ResourcesVersion {
    fn cmp(&self, other:&Self) -> Ordering {
        self.comparing.cmp(&other.comparing)
    }
}

impl PartialEq for ResourcesVersion {
    fn eq(&self, other:&Self) -> bool {
        self.comparing==other.comparing
    }
}

impl<'a> Root<'_> {
    pub fn new(base_path:&'a str, version:&'a str) -> Root<'a> {
        Root {base_path:Path::new(base_path), version:ResourcesVersion::new(version)}
    }

    pub fn read_data_or_default(&self,file:&str) -> Vec<u8> {
        fs::read(self.base_path.join(file).into_os_string()).unwrap_or_default()
    }

    pub fn file_exists(&self,file:&str) -> bool {
        self.base_path.join(file).exists()
    }

    pub fn version(&self) -> &ResourcesVersion {
        &self.version
    }

    pub fn v3_directory_file(&self) -> Result<String,&'static str> {
        if let Ok(iter) = fs::read_dir(self.base_path) {
            for a in iter {
                if let Ok(entry) = a {
                    if let Ok(name) = entry.file_name().into_string() {
                        if name.ends_with("DIR") {
                            return Ok(name.clone());
                        }
                    }
                }
            }
        }
        return Err("Failed to locate V3 Directory Resource");
    }

    fn fetch_volume_name(&self,entry:&ResourceDirectoryEntry) -> Result<String,&'static str> {
        let vol_name = format!("VOL.{}",entry.volume);
        if let Ok(iter) = fs::read_dir(self.base_path) {
            for a in iter {
                if let Ok(entry) = a {
                    if let Ok(name) = entry.file_name().into_string() {
                        if name.ends_with(vol_name.as_str()) {
                            return Ok(name.clone());
                        }
                    }
                }
            }
        }
        return Err("Failed to locate V3 Directory Resource");

    }

    pub fn fetch_volume(&self,entry:&ResourceDirectoryEntry) -> Vec<u8> {
        if let Ok(name) = self.fetch_volume_name(entry) {
            return self.read_data_or_default(name.as_str());
        }
        Vec::new()
    }
}

pub enum ResourceType {
    Words,
    Pictures,
    Logic,
    Objects,
    Views,
}

pub enum ResourceCompression {
    None,
    LZW,
    Picture,
}

/// Represents an entry in a Directory Resource in AGI
pub struct ResourceDirectoryEntry {
    pub volume:u8,
    pub position:u32,
    pub compression:ResourceCompression,
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

//todo get Words,Objects,etc
impl ResourceDirectory {

    pub fn new(root:&Root,resource_type:ResourceType) -> Result<ResourceDirectory, &'static str> {

        let directory_name = match resource_type {
            ResourceType::Objects | ResourceType::Words => panic!("We should never request resource directory for these resource types"),
            ResourceType::Pictures => "PICDIR",
            ResourceType::Views => "VIEWDIR",
            ResourceType::Logic => "LOGDIR",
        };
        if root.file_exists(directory_name) {
            let bytes = root.read_data_or_default(directory_name);
            return ResourceDirectory::new_v2(bytes);
        }

        // Presumably we are looking at a v3 directory resource
        if let Ok(v3) = root.v3_directory_file() {
            return ResourceDirectory::new_v3(root.read_data_or_default(v3.as_str()), resource_type);
        }
        Err("Oh dear")
    }

    fn new_v2(bytes: Vec<u8>) -> Result<ResourceDirectory, &'static str> {

        let mut entries = Vec::new();
        let mut bytes = bytes.into_iter();

        while let Some(b) = bytes.next() {
            let volume = b>>4;
            let position:u32 = (b&0xF).into();

            if let Some(b) = bytes.next() {
                let t:u32 = b.into();
                let position:u32 = (position<<8) + t;

                if let Some(b) = bytes.next() {
                    let t:u32 = b.into();
                    let position:u32 = (position<<8) + t;
                    let compression = ResourceCompression::None;
                    entries.push(ResourceDirectoryEntry { volume, position, compression});
                } else {
                    return Err("Input bytes are not made up of triples (size % 3 != 0)");
                }
            } else {
                return Err("Input bytes are not made up of triples (size % 3 != 0)");
            }
        }
        Ok(ResourceDirectory{entries})
    }

    fn new_v3(bytes: Vec<u8>,resource_type:ResourceType) -> Result<ResourceDirectory, &'static str> {

        let mut entries = Vec::new();
        let mut bytes = bytes.into_iter();

        // Get correct header entry
        let logic_offset:u16;
        let picture_offset:u16;
        let view_offset:u16;
        let sound_offset:u16;
        if let Some(lo) = bytes.next() {
            if let Some(hi) = bytes.next() {
                logic_offset=((hi as u16)<<8)+(lo as u16);
            } else {
                return Err("Expected logic hi offset");
            }
        } else {
            return Err("Exected logic lo offset");
        }
        if let Some(lo) = bytes.next() {
            if let Some(hi) = bytes.next() {
                picture_offset=((hi as u16)<<8)+(lo as u16);
            } else {
                return Err("Expected picture hi offset");
            }
        } else {
            return Err("Exected picture lo offset");
        }
        if let Some(lo) = bytes.next() {
            if let Some(hi) = bytes.next() {
                view_offset=((hi as u16)<<8)+(lo as u16);
            } else {
                return Err("Expected view hi offset");
            }
        } else {
            return Err("Exected view lo offset");
        }
        if let Some(lo) = bytes.next() {
            if let Some(hi) = bytes.next() {
                sound_offset=((hi as u16)<<8)+(lo as u16);
            } else {
                return Err("Expected sound hi offset");
            }
        } else {
            return Err("Exected sound lo offset");
        }

        let (skip,take) = match resource_type {
            ResourceType::Words | ResourceType::Objects => panic!("We should never request resource directory for these resource types"),
            ResourceType::Logic => (logic_offset-8,picture_offset-logic_offset),
            ResourceType::Pictures => (picture_offset-8,view_offset-picture_offset),
            ResourceType::Views => (view_offset-8,sound_offset-view_offset),
        };
        let mut bytes = bytes.skip(skip as usize).take(take as usize);
        while let Some(b) = bytes.next() {
            let volume = b>>4;
            let position:u32 = (b&0xF).into();

            if let Some(b) = bytes.next() {
                let t:u32 = b.into();
                let position:u32 = (position<<8) + t;

                if let Some(b) = bytes.next() {
                    let t:u32 = b.into();
                    let position:u32 = (position<<8) + t;
                    let compression = match resource_type {
                        ResourceType::Pictures => ResourceCompression::Picture,
                        _ => ResourceCompression::LZW,
                    };
                    entries.push(ResourceDirectoryEntry { volume, position, compression});
                } else {
                    return Err("Input bytes are not made up of triples (size % 3 != 0)");
                }
            } else {
                return Err("Input bytes are not made up of triples (size % 3 != 0)");
            }
        }
        Ok(ResourceDirectory{entries})
    }

    pub fn get(&self,index: usize) -> Option<&ResourceDirectoryEntry> {
        self.entries.get(index)
    }



}
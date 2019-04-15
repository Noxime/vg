//! # Filesystem API for immutable assets
//! This module contains an API to load asset packs into your game. 
//! 
//! # Usage
//! To use asset packing, you have to add a `build.rs` to your crate root and place the following line in it:
//! ```rust
//! use kea::assets::generate_asset_packs;
//! fn main() {
//!     generate_asset_pack("assets/", "assets.keapack");
//! }
//! ```
//! _remember to add kea into your `[build-dependencies]` too_
//! 
//! Then in your code 
//! ```rust
//! const ASSETS: Assets = asset_pack!("assets.keapack");
//! 
//! // ...
//! 
//! let config_str = ASSETS::str("config.toml")?;
//! let splash = ASSETS::assets("textures")?.bin("splash.png")?;
//! ```
//! 
//! # Asset pack internal format
//! This is something you should not need to worry about, but I will document it briefly here anyway.
//! 
//! Currently asset packs in kea (`.keapack`) are quite a basic file format, and hopefully will stay that way.
//! 
//! ## Asset pack header
//! | Field        | Offset           | Type          | Description                                                  |
//! |--------------|------------------|---------------|--------------------------------------------------------------|
//! | Size         | 0x0              | u64           | The complete size of the asset pack, including data + header |
//! | Count        | 0x8              | u64           | The number of Data headers in this asset pack                |
//! | Offset table | 0x16             | [u64; Count]  | Offsets of Data headers relative to the start of this header |
//! | Data         | 0x16 + Count*0x8 | [Data; Count] | All the files contained in this Asset pack                   |
//! | Name length  | Size-Name.len -1 | u8            | How long the name is                                         |
//! | Name         | Size - Name.len  | str           | The name of the asset pack                                   |
//! 
//! ## Data header
//! | Field       | Offset        | Type         | Description                                                                |
//! |-------------|---------------|--------------|----------------------------------------------------------------------------|
//! | Size        | 0x0           | u64          | The size of this data header                                               |
//! | Type        | 0x8           | u8           | `0` means this file is another asset pack, `1` means this is a binary file |
//! | Length      | 0x9           | u64          | The size of the binary data in this data header                            |
//! | Data        | 0x17          | [u8; Length] | The actual binary data in this header (Note: can be another asset pack)    |
//! | Name length | 0x18 + Length | u8           | How long the name is                                                       |
//! | Name        | 0x19 + Length | str          | The name of the asset pack                                                 |

use std::path::Path;

// # Asset header
// size: u64
// count: u64
// file_offset_table: [u64; count]
// data: [Data; count]
// name_length: u8
// name: str

// # Data header
// size: u64
// type: u8 // 0 => asset 1 => binary
// data size: u64
// data [u8; datasize]
// name_length: u8
// name: str

#[derive(PartialEq)]
enum DataType {
    Assets,
    Binary,
}

struct Data<'a> {
    data: &'a [u8]
}

impl<'a> Data<'a> {
    pub fn size(&self) -> u64 {
        u64::from_le_bytes([
            self.data[0],
            self.data[1],
            self.data[2],
            self.data[3],
            self.data[4],
            self.data[5],
            self.data[6],
            self.data[7],
        ])
    }

    fn kind(&self) -> DataType {
        match self.data[8] {
            0 => DataType::Assets,
            1 => DataType::Binary,
            v => panic!("Unknown data type {}", v),
        }
    }

    pub fn data_size(&self) -> u64 {
        u64::from_le_bytes([
            self.data[9],
            self.data[10],
            self.data[11],
            self.data[12],
            self.data[13],
            self.data[14],
            self.data[15],
            self.data[16],
        ])
    }

    fn data(&self) -> &'a [u8] {
        &self.data[17 .. (17 + self.data_size()) as usize]
    }

    fn name_length(&self) -> u8 {
        self.data[17 + self.data_size() as usize]
    }

    fn name(&self) -> &'a str {
        let offset = 18 + self.data_size() as usize;
        std::str::from_utf8(&self.data[offset..offset + self.name_length() as usize]).expect("Asset pack data had invalid name")
    }

}

/// [`Assets`] provides an API for loading data from asset packs
/// 
/// The only way currently to create [`Assets`] is to use the [`asset_pack!`] macro. See its 
/// documentation for more details.
/// 
/// # Example
/// ```rust
/// const TEXTURES: Assets = asset_pack!("textures_new.keapack");
/// let image = decode_png(TEXTURES.assets("x512")?.binary()?)?;
/// // do whatever with your image :)
/// ```
pub struct Assets<'a> {
    #[doc(hidden)]
    pub data: &'a [u8]
}

/// Represents an error when you try to read an non-existing file
pub struct NotFound;

impl<'a> Assets<'a> {
    /// The byte size of the asset pack
    /// 
    /// Note: This is not the combined size of the files in the asset pack. This size also contains the internal asset
    /// pack header data
    pub fn size(&self) -> u64 {
        u64::from_le_bytes([
            self.data[0],
            self.data[1],
            self.data[2],
            self.data[3],
            self.data[4],
            self.data[5],
            self.data[6],
            self.data[7],
        ])
    }

    /// How many files or asset packs this asset pack contains
    pub fn count(&self) -> u64 {
        u64::from_le_bytes([
            self.data[8],
            self.data[9],
            self.data[10],
            self.data[11],
            self.data[12],
            self.data[13],
            self.data[14],
            self.data[15],
        ])
    }

    fn file_table(&self) -> &[u64] {
        let count = self.count() as usize;
        unsafe { std::slice::from_raw_parts(self.data.as_ptr().offset(16) as *const u64, count) }
    }

    fn file_offset(&self, index: usize) -> u64 {
        self.file_table()[index]
    }

    fn data(&self, index: usize) -> Data<'a> {
        Data { data: &self.data[self.file_offset(index) as usize .. ] }
    }

    // The name of this asset pack
    pub fn name(&self) -> &'a str {
        let count = self.count();
        let offset: usize = match count {
            0 => 17,
            count => {
                let offset = self.file_offset(count as usize - 1);
                self.data(count as usize - 1).size() as usize + offset as usize + 1
            }
        };
        let length = self.data[offset] as usize;
        let offset = offset + 1;
        std::str::from_utf8(&self.data[offset..offset + length]).expect("Asset pack had invalid name")
    }

    // Find a binary file in this asset pack
    // 
    // Note: This function does not recurse, use [`assets`](Assets::assets) to access nested asset packs
    pub fn binary(&self, name: &str) -> Result<&'a [u8], NotFound> {
        for i in 0 .. self.count() {
            let data = self.data(i as usize);
            if data.name() == name && data.kind() == DataType::Binary {
                return Ok(data.data())
            }
        }
        Err(NotFound)
    }

    /// Find a nested asset pack
    pub fn assets(&self, name: &str) -> Result<Assets<'a>, NotFound> {
        for i in 0 .. self.count() {
            let data = self.data(i as usize);
            if data.name() == name && data.kind() == DataType::Assets {
                return Ok(Assets { data: data.data() })
            }
        }
        Err(NotFound)
    }

    pub fn all_binaries(&self) -> Vec<(&'a str, &'a [u8])> {
        let mut buf = vec![];
        for i in 0 .. self.count() {
            let data = self.data(i as usize);
            if data.kind() == DataType::Binary {
                buf.push((data.name(), data.data()))
            }
        }
        buf
    }

    pub fn all_assets(&self) -> Vec<Assets<'a>> {
        let mut buf = vec![];
        for i in 0 .. self.count() {
            let data = self.data(i as usize);
            if data.kind() == DataType::Assets {
                buf.push(Assets { data: data.data() })
            }
        }
        buf
    }
}

fn recurse_asset_packs(path: &Path, root: bool) -> Vec<u8> {
    assert!(Path::new(path).exists());

    let (mut data, kind) = if path.is_dir() {
        // asset pack
        let mut buf = vec![];
        let mut count: u64 = 0;
        let mut offsets = vec![];
        let entries: Vec<std::fs::DirEntry> = std::fs::read_dir(path).expect("Asset folder read failed").filter_map(|e| e.ok()).collect();
        let mut data_offset = 8 * entries.len() as u64 + 16;
        let mut data = vec![];
        for entry in &entries {
            count += 1;
            let mut entry = recurse_asset_packs(&entry.path(), false);
            offsets.push(data_offset);
            data_offset += entry.len() as u64;
            data.append(&mut entry);
        }

        let name = path.file_name().expect("No file name in path")
            .to_str().expect("File name is not string").as_bytes();
        let size = 16 + 8 * offsets.len() + data.len() + name.len();

        buf.append(&mut vec![
            (size >> 0) as u8,
            (size >> 8) as u8,
            (size >> 16) as u8,
            (size >> 24) as u8,
            (size >> 32) as u8,
            (size >> 40) as u8,
            (size >> 48) as u8,
            (size >> 56) as u8,
        ]);
        buf.append(&mut vec![
            (count >> 0) as u8,
            (count >> 8) as u8,
            (count >> 16) as u8,
            (count >> 24) as u8,
            (count >> 32) as u8,
            (count >> 40) as u8,
            (count >> 48) as u8,
            (count >> 56) as u8,
        ]);

        for offset in offsets {
            buf.append(&mut vec![
                (offset >> 0) as u8,
                (offset >> 8) as u8,
                (offset >> 16) as u8,
                (offset >> 24) as u8,
                (offset >> 32) as u8,
                (offset >> 40) as u8,
                (offset >> 48) as u8,
                (offset >> 56) as u8,
            ]);
        }

        buf.append(&mut data);

        buf.push(name.len() as u8);
        buf.append(&mut Vec::from(name));

        (buf, 0)
    } else {
        // binary
        (std::fs::read(path).expect("Asset file read failed"), 1)
    };

    if root {
        return data;
    }

    let name = path.file_name().expect("No file name in path")
        .to_str().expect("File name is not string").as_bytes();
    let size = 8 + 1 + 8 + data.len() + name.len();

    let mut buf = vec![];
    buf.append(&mut vec![
        (size >> 0) as u8,
        (size >> 8) as u8,
        (size >> 16) as u8,
        (size >> 24) as u8,
        (size >> 32) as u8,
        (size >> 40) as u8,
        (size >> 48) as u8,
        (size >> 56) as u8,
    ]);

    buf.push(kind);

    buf.append(&mut vec![
        (data.len() >> 0) as u8,
        (data.len() >> 8) as u8,
        (data.len() >> 16) as u8,
        (data.len() >> 24) as u8,
        (data.len() >> 32) as u8,
        (data.len() >> 40) as u8,
        (data.len() >> 48) as u8,
        (data.len() >> 56) as u8,
    ]);
    buf.append(&mut data);

    buf.push(name.len() as u8);
    buf.append(&mut Vec::from(name));
    buf
}


pub fn generate_asset_pack(path: &str, name: &str) {
    let pack = recurse_asset_packs(Path::new(path), true);
    if pack.len() > 1024 * 1024 * 128 {
        println!("cargo:warning=Asset pack `{}` is very large ({:.2} Mb), consider splitting it into smaller packs", name, pack.len() as f64 / 1024.0 / 1024.0);
    }
    let _ = std::fs::write(format!("{}/{}", std::env::var("OUT_DIR").unwrap(), name), pack).expect("Asset pack write failed");
}

/// This macro includes a built asset pack into your game
/// 
/// Note: This macro will fail to compile with a cryptic message `environment variable OUT_DIR is not set` if your 
/// game does not have a `build.rs` file.
#[macro_export]
macro_rules! asset_pack {
    ($path:expr) => {
        crate::assets::Assets { data: include_bytes!(concat!(env!("OUT_DIR"), "/", $path)) }
    };
}
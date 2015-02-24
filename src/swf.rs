use std::old_io::{ Reader, IoResult, IoError, InvalidInput };
use std::old_io::util::LimitReader;
use std::num;

#[cfg(feature = "zlib")]
use flate2::reader::ZlibDecoder;

#[derive(Debug, Copy, Clone)]
pub struct Magic {
    pub compression: Compression,
    pub version: u8,
    pub size: u32,
}

#[derive(Debug, Clone)]
pub struct Header {
    pub rectangle: Vec<u8>,
    pub frame_rate: u16,
    pub frame_count: u16,
}

impl Magic {
    pub fn read<R : Reader>(reader: &mut R) -> IoResult<Self> {
        let magic = try!(reader.read_be_u32());

        if magic & 0x00ffff00 != 0x00575300 { // 'CWS\version' or 'FWS\version'
            return Err(IoError { kind: InvalidInput, desc: "unknown SWF magic", detail: None })
        }

        let compression = (magic >> 24) as u8;
        let version = (magic & 0xff) as u8;
        let size = try!(reader.read_le_u32());

        let compression = match num::from_u8(compression) {
            Some(c) => c,
            None => return Err(IoError { kind: InvalidInput, desc: "unknown SWF compression magic", detail: None })
        };

        Ok(Magic {
            compression: compression,
            version: version,
            size: size,
        })
    }

    pub fn len(&self) -> usize {
        8
    }

    pub fn reader<'a, R : Reader + 'a>(&self, reader: R) -> Box<Reader + 'a> {
        let len = self.size as usize - self.len();
        match self.compression {
            Compression::None => Box::new(LimitReader::new(reader, len)),

            #[cfg(feature = "zlib")]
            Compression::Zlib => Box::new(LimitReader::new(ZlibDecoder::new(reader), len)),

            #[cfg(not(feature = "zlib"))]
            Compression::Zlib => panic!("use `zlib' feature to support compressed SWF"),

            _ => unimplemented!()
        }
    }
}

impl Header {
    pub fn read<R : Reader>(reader: &mut R) -> IoResult<Self> {
        let rect_size = (4 * (try!(reader.read_u8()) >> 3) as usize - 3 + 7) / 8;
        let rectangle = try!(reader.read_exact(rect_size));
        let frame_rate = try!(reader.read_le_u16());
        let frame_count = try!(reader.read_le_u16());

        Ok(Header {
            rectangle: rectangle,
            frame_rate: frame_rate,
            frame_count: frame_count
        })
    }

    pub fn len(&self) -> usize {
        5 + self.rectangle.len()
    }
}

#[derive(Debug, Copy)]
pub struct Tag {
    pub kind: TagKind,
    pub length: u32,
}

impl Tag {
    pub fn read<R : Reader>(reader: &mut R) -> IoResult<Self> {
        let value = try!(reader.read_le_u16());
        let kind = match num::from_u16(value >> 6) {
            Some(k) => k,
            None => return Err(IoError { kind: InvalidInput, desc: "unknown tag", detail: None })
        };

        let value = value & 0x3f;
        let length = match value {
            0x3f => try!(reader.read_le_u32()),
            _ => value as u32
        };

        Ok(Tag {
            kind: kind,
            length: length
        })
    }

    pub fn write<W : Writer>(&self, writer: &mut W) -> IoResult<()> {
        let kind = (self.kind as u16) << 6;
        match self.length {
            0...0x3e => try!(writer.write_le_u16(kind | self.length as u16)),
            _ => {
                try!(writer.write_le_u16(kind | 0x3f));
                try!(writer.write_le_u32(self.length));
            }
        }

        Ok(())
    }
}

#[derive(Debug, FromPrimitive, Copy, Clone, PartialEq, Eq)]
pub enum Compression {
    Zlib = 'C' as isize,
    Lzma = 'Z' as isize,
    None = 'F' as isize
}

#[derive(Debug, FromPrimitive, Copy, Clone, PartialEq, Eq)]
pub enum TagKind {
    End                          = 0,
    ShowFrame                    = 1,
    DefineShape                  = 2,
    PlaceObject                  = 4,
    RemoveObject                 = 5,
    DefineBits                   = 6,
    DefineButton                 = 7,
    JPEGTables                   = 8,
    SetBackgroundColor           = 9,
    DefineFont                   = 10,
    DefineText                   = 11,
    DoAction                     = 12,
    DefineFontInfo               = 13,
    DefineSound                  = 14,
    StartSound                   = 15,
    DefineButtonSound            = 17,
    SoundStreamHead              = 18,
    SoundStreamBlock             = 19,
    DefineButsLossless           = 20,
    DefineBitsJPEG2              = 21,
    DefineShape2                 = 22,
    DefineButtonCxform           = 23,
    Protect                      = 24,
    PlaceObject2                 = 26,
    RemoveObject2                = 28,
    DefineShape3                 = 32,
    DefineText2                  = 33,
    DefineButton2                = 34,
    DefineBitsJPEG3              = 35,
    DefineBitsLossless2          = 36,
    DefineEditText               = 37,
    DefineSprite                 = 39,
    FrameLabel                   = 43,
    SoundStreamHead2             = 45,
    DefineMorphShape             = 46,
    DefineFont2                  = 48,
    ExportAssets                 = 56,
    ImportAssets                 = 57,
    EnableDebugger               = 58,
    DoInitAction                 = 59,
    DefineVideoStream            = 60,
    VideoFrame                   = 61,
    DefineFontInfo2              = 62,
    EnableDebugger2              = 64,
    ScriptLimits                 = 65,
    SetTabIndex                  = 66,
    FileAttributes               = 69,
    PlaceObject3                 = 70,
    ImportAssets2                = 71,
    DefineFontAlignZones         = 73,
    CSMTextSettings              = 74,
    DefineFont3                  = 75,
    SymbolClass                  = 76,
    Metadata                     = 77,
    DefineScalingGrid            = 78,
    DoABC                        = 82,
    DefineShape4                 = 83,
    DefineMorphShape2            = 84,
    DefineSceneAndFrameLabelData = 86,
    DefineBinaryData             = 87,
    DefineFontName               = 88,
    StartSound2                  = 89,
    DefineBitsJPEG4              = 90,
    DefineFont4                  = 91,
    EnableTelemetry              = 93
}

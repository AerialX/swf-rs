#![feature(core, old_io, collections, std_misc)]

#[cfg(feature = "zlib")]
extern crate flate2;

mod swf;
pub use self::swf::{ Compression, Header, Magic, Tag, TagKind };

pub mod util;
pub mod avm1;

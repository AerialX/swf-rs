use std::old_io::{ Reader, ByRefReader, IoResult, IoError, InvalidInput, EndOfFile };
use std::old_io::util::{ LimitReader, NullWriter, copy };
use swf::{ Tag, TagKind };
use std::mem::replace;
use std::num;

pub struct SoundStream<R> {
    pub format: SoundFormat,
    pub samples: u32,
    pub seek: u16,
    pub id: u16,
    pub start_frame: usize,
    started: bool,

    reader: IoResult<LimitReader<R>>,
}

#[derive(Debug, FromPrimitive, Copy, Clone, PartialEq, Eq)]
pub enum CodecKind {
    MP3 = 2
}

#[derive(Debug, Copy, Clone)]
pub struct SoundFormat {
    format: u8,
    sample_rate: u16,
    codec: CodecKind
}

impl SoundFormat {
    pub fn new(format: u8) -> Option<Self> {
        Some(SoundFormat {
            format: format,
            sample_rate: match (format >> 2) & 0x3 {
                0 => 5500,
                1 => 11000,
                2 => 22050,
                3 => 44100,
                _ => unreachable!()
            },
            codec: if let Some(c) = num::from_u8((format >> 4) & 0x0f) { c } else { return None }
        })
    }
}

impl<R : Reader> SoundStream<R> {
    pub fn new(tag: Tag, reader: R) -> IoResult<Self> {
        let format;
        let samples;
        let seek;
        let id;

        let codec_error = IoError { kind: InvalidInput, desc: "unsupported audio codec", detail: None };

        let mut reader = LimitReader::new(reader, tag.length as usize);
        match tag.kind {
            TagKind::SoundStreamHead | TagKind::SoundStreamHead2 => {
                try!(reader.read_u8());
                format = try!(SoundFormat::new(try!(reader.read_u8())).ok_or(codec_error));
                try!(reader.read_le_u16()); // average frame samples
                seek = try!(reader.read_le_u16());
                try!(SoundStream::drain(&mut reader));
                samples = 0;
                id = 0;
            },
            TagKind::DefineSound => {
                id = try!(reader.read_le_u16());
                format = try!(SoundFormat::new(try!(reader.read_u8())).ok_or(codec_error));
                samples = try!(reader.read_le_u32());
                seek = try!(reader.read_le_u16());
            },
            _ => return Err(IoError { kind: InvalidInput, desc: "audio tag expected", detail: None })
        }


        Ok(SoundStream {
            reader: Ok(reader),
            format: format,
            samples: samples,
            seek: seek,
            id: id,
            start_frame: 0,
            started: false
        })
    }

    pub fn into_inner(self) -> R {
        self.reader.unwrap().into_inner()
    }

    fn reader(&mut self) -> IoResult<&mut LimitReader<R>> {
        match self.reader.as_mut() {
            Ok(r) => Ok(r),
            Err(err) => Err(err.clone())
        }
    }

    fn drain(reader: &mut LimitReader<R>) -> IoResult<()> {
        copy(reader, &mut NullWriter)
    }

    fn process(&mut self, mut reader: R) -> IoResult<LimitReader<R>> {
        let mut limit;
        loop {
            let tag = try!(Tag::read(&mut reader));
            let mut tagreader = LimitReader::new(reader.by_ref(), tag.length as usize);
            match tag.kind {
                TagKind::SoundStreamBlock => {
                    self.started = true;
                    self.samples += try!(tagreader.read_le_u16()) as u32;
                    try!(tagreader.read_le_u16()); // seek samples
                    limit = tagreader.limit();
                    break;
                },
                TagKind::End => return Err(IoError { kind: EndOfFile, desc: "end swf tag", detail: None }),
                TagKind::ShowFrame => self.start_frame += 1,
                _ => ()
            }
            try!(SoundStream::drain(&mut tagreader));
        }

        return Ok(LimitReader::new(reader, limit));
    }
}

impl<R : Reader> Reader for SoundStream<R> {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        loop {
            return match try!(self.reader()).read(buf) {
                Err(ref err) if err.kind == EndOfFile => {
                    let err = err.clone();
                    let reader = replace(&mut self.reader, Err(err)).unwrap().into_inner();
                    let reader = self.process(reader);
                    let _ = replace(&mut self.reader, reader);
                    match self.reader.as_ref() {
                        Err(err) => Err(err.clone()),
                        _ => continue
                    }
                },
                Err(err) => Err(err),
                Ok(sz) => Ok(sz)
            }
        }
    }
}

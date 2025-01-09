use std::io::Read;

use byteorder::{LittleEndian, ReadBytesExt};

use lzma_rust::{lzma2_get_memery_usage, LZMA2Reader, LZMAReader};

use crate::{archive::SevenZMethod, error::Error, folder::Coder};

pub enum Decoder<R: Read> {
    Copy(R),
    Lzma(LZMAReader<R>),
    Lzma2(LZMA2Reader<R>),
}

impl<R: Read> Read for Decoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Decoder::Copy(r) => r.read(buf),
            Decoder::Lzma(r) => r.read(buf),
            Decoder::Lzma2(r) => r.read(buf),
        }
    }
}

pub fn add_decoder<I: Read>(
    input: I,
    uncompressed_len: usize,
    coder: &Coder,
    max_mem_limit_kb: usize,
) -> Result<Decoder<I>, Error> {
    let method = SevenZMethod::by_id(coder.decompression_method_id());
    let method = if let Some(m) = method {
        m
    } else {
        return Err(Error::Unsupported(format!(
            "{:?}",
            coder.decompression_method_id()
        )));
    };
    match method.id() {
        SevenZMethod::ID_COPY => Ok(Decoder::Copy(input)),
        SevenZMethod::ID_LZMA => {
            let dict_size = get_lzma_dic_size(coder)?;
            if coder.properties.is_empty() {
                return Err(Error::Other("LZMA properties too short".into()));
            }
            let props = coder.properties[0];
            let lz =
                LZMAReader::new_with_props(input, uncompressed_len as _, props, dict_size, None)
                    .map_err(Error::io)?;
            Ok(Decoder::Lzma(lz))
        }
        SevenZMethod::ID_LZMA2 => {
            let dic_size = get_lzma2_dic_size(coder)?;
            let mem_size = lzma2_get_memery_usage(dic_size) as usize;
            if mem_size > max_mem_limit_kb {
                return Err(Error::MaxMemLimited {
                    max_kb: max_mem_limit_kb,
                    actual_kb: mem_size,
                });
            }
            let lz = LZMA2Reader::new(input, dic_size, None);
            Ok(Decoder::Lzma2(lz))
        }
        _ => Err(Error::Unsupported(method.name().to_string())),
    }
}

#[inline]
fn get_lzma2_dic_size(coder: &Coder) -> Result<u32, Error> {
    if coder.properties.is_empty() {
        return Err(Error::other("LZMA2 properties too short"));
    }
    let dict_size_bits = 0xff & coder.properties[0] as u32;
    if (dict_size_bits & (!0x3f)) != 0 {
        return Err(Error::other("Unsupported LZMA2 property bits"));
    }
    if dict_size_bits > 40 {
        return Err(Error::other("Dictionary larger than 4GiB maximum size"));
    }
    if dict_size_bits == 40 {
        return Ok(0xFFFFFFFF);
    }
    let size = (2 | (dict_size_bits & 0x1)) << (dict_size_bits / 2 + 11);
    Ok(size)
}

#[inline]
fn get_lzma_dic_size(coder: &Coder) -> Result<u32, Error> {
    if coder.properties.len() < 5 {
        return Err(Error::other("LZMA properties too short"));
    }
    let mut props = &coder.properties[1..5];
    props.read_u32::<LittleEndian>().map_err(Error::io)
}

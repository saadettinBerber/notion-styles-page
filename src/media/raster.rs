use std::io::Cursor;
use std::num::NonZeroU64;

use crate::PageError;

const MAX_IMAGE_PIXELS: u64 = 89_478_485;
const MAX_RASTER_BYTES: usize = 512 * 1024 * 1024;
const PNG_SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";
const PNG_END: &[u8] = b"\0\0\0\0IEND\xae\x42\x60\x82";
const GIF_END: &[u8] = b"\0;";

/// MIME type and encoded canvas dimensions of a validated raster image.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageInfo {
    pub media_type: &'static str,
    pub width: u32,
    pub height: u32,
}

/// Validate a complete PNG, JPEG, GIF or WebP, including animation frames.
pub fn inspect_image(content: &[u8]) -> Result<ImageInfo, PageError> {
    let (media_type, dimensions) = match content {
        png if png.starts_with(PNG_SIGNATURE) && png.ends_with(PNG_END) => {
            ("image/png", inspect_png(png))
        }
        jpeg if jpeg.starts_with(b"\xff\xd8") => ("image/jpeg", inspect_jpeg(jpeg)),
        gif if (gif.starts_with(b"GIF87a") || gif.starts_with(b"GIF89a"))
            && gif.ends_with(GIF_END) =>
        {
            ("image/gif", inspect_gif(gif))
        }
        webp if webp.starts_with(b"RIFF") && webp.get(8..12) == Some(b"WEBP") => {
            ("image/webp", inspect_webp(webp))
        }
        _ => return Err(invalid_image()),
    };
    let (width, height) = dimensions.ok_or_else(invalid_image)?;
    Ok(ImageInfo {
        media_type,
        width,
        height,
    })
}

fn invalid_image() -> PageError {
    PageError::Media("Tam bir PNG, JPEG, GIF veya WebP görseli olmalı.".into())
}

fn checked_dimensions(width: u32, height: u32) -> Option<(u32, u32)> {
    let pixels = u64::from(width) * u64::from(height);
    (pixels > 0 && pixels <= MAX_IMAGE_PIXELS).then_some((width, height))
}

fn inspect_png(content: &[u8]) -> Option<(u32, u32)> {
    let mut options = png::DecodeOptions::default();
    options.set_skip_ancillary_crc_failures(false);
    options.set_ignore_adler32(false);
    let mut decoder = png::Decoder::new_with_options(Cursor::new(content), options);
    let header = decoder.read_header_info().ok()?;
    let dimensions = checked_dimensions(header.width, header.height)?;
    let mut reader = decoder.read_info().ok()?;
    let frame_count = png_animation_frame_count(&content[PNG_SIGNATURE.len()..])?;
    let frames_after_default =
        frame_count.checked_sub(u32::from(reader.info().frame_control.is_some()))?;
    read_png_frame(&mut reader)?;
    for _ in 0..frames_after_default {
        reader.next_frame_info().ok()?;
        read_png_frame(&mut reader)?;
    }
    reader.finish().ok()?;
    Some(dimensions)
}

fn read_png_frame(reader: &mut png::Reader<Cursor<&[u8]>>) -> Option<()> {
    while reader.next_row().ok()?.is_some() {}
    Some(())
}

// The decoder iterates only as many frames as acTL declares. Check the physical
// fcTL count so extra frames cannot bypass decoding and validation.
fn png_animation_frame_count(mut chunks: &[u8]) -> Option<u32> {
    let mut declared = None;
    let mut frames = 0_u32;
    loop {
        let len = u32::from_be_bytes(chunks.get(..4)?.try_into().ok()?);
        let end = 12_usize.checked_add(usize::try_from(len).ok()?)?;
        let (chunk, rest) = chunks.split_at_checked(end)?;
        match &chunk[4..8] {
            b"acTL" => {
                let data = chunk.get(8..end - 4)?;
                let count = u32::from_be_bytes(data.get(..4)?.try_into().ok()?);
                if count == 0 || declared.replace(count).is_some() {
                    return None;
                }
            }
            b"fcTL" => frames = frames.checked_add(1)?,
            b"IEND" => return (frames == declared.unwrap_or(0)).then_some(frames),
            _ => {}
        }
        chunks = rest;
    }
}

fn inspect_jpeg(content: &[u8]) -> Option<(u32, u32)> {
    let mut decoder = jpeg_decoder::Decoder::new(content);
    decoder.set_max_decoding_buffer_size(MAX_RASTER_BYTES);
    decoder.read_info().ok()?;
    let info = decoder.info()?;
    let dimensions = checked_dimensions(u32::from(info.width), u32::from(info.height))?;
    decoder.decode().ok()?;
    Some(dimensions)
}

fn inspect_gif(content: &[u8]) -> Option<(u32, u32)> {
    let mut options = gif::DecodeOptions::new();
    options.set_memory_limit(gif::MemoryLimit::Bytes(NonZeroU64::new(MAX_IMAGE_PIXELS)?));
    let mut decoder = options.read_info(content).ok()?;
    let dimensions = checked_dimensions(u32::from(decoder.width()), u32::from(decoder.height()))?;
    decoder.read_next_frame().ok()??;
    while decoder.read_next_frame().ok()?.is_some() {}
    Some(dimensions)
}

fn inspect_webp(content: &[u8]) -> Option<(u32, u32)> {
    let riff_size = u32::from_le_bytes(content.get(4..8)?.try_into().ok()?);
    let riff_end = usize::try_from(riff_size).ok()?.checked_add(8)?;
    let content = content.get(..riff_end)?;
    let frames = webp_animation_frame_count(content.get(12..)?)?;
    let mut decoder = image_webp::WebPDecoder::new(Cursor::new(content)).ok()?;
    decoder.set_memory_limit(MAX_RASTER_BYTES);
    let (width, height) = decoder.dimensions();
    let dimensions = checked_dimensions(width, height)?;
    if frames != decoder.num_frames() {
        return None;
    }
    let mut frame = vec![0; decoder.output_buffer_size()?];
    if decoder.is_animated() {
        for _ in 0..decoder.num_frames() {
            decoder.read_frame(&mut frame).ok()?;
        }
    } else {
        decoder.read_image(&mut frame).ok()?;
    }
    Some(dimensions)
}

fn webp_animation_frame_count(mut chunks: &[u8]) -> Option<u32> {
    let mut frames = 0_u32;
    while !chunks.is_empty() {
        let (kind, data, rest) = riff_chunk(chunks)?;
        if kind == b"ANMF" {
            check_webp_frame(data)?;
            frames = frames.checked_add(1)?;
        }
        chunks = rest;
    }
    Some(frames)
}

fn riff_chunk(chunks: &[u8]) -> Option<(&[u8], &[u8], &[u8])> {
    let kind = chunks.get(..4)?;
    let len = usize::try_from(u32::from_le_bytes(chunks.get(4..8)?.try_into().ok()?)).ok()?;
    let end = 8_usize.checked_add(len)?;
    let next = end.checked_add(len % 2)?;
    Some((kind, chunks.get(8..end)?, chunks.get(next..)?))
}

fn check_webp_frame(frame: &[u8]) -> Option<()> {
    let (kind, _, next) = riff_chunk(frame.get(16..)?)?;
    if kind != b"ALPH" {
        return Some(());
    }
    // The animated ALPH decoder allocates from ANMF but indexes using the
    // VP8 bitstream dimensions. Reject mismatched headers before decoding.
    let (kind, data, _) = riff_chunk(next)?;
    let size_24 = |bytes: &[u8]| u32::from_le_bytes([bytes[0], bytes[1], bytes[2], 0]) + 1;
    let vp8_width = u16::from_le_bytes(data.get(6..8)?.try_into().ok()?) & 0x3fff;
    let vp8_height = u16::from_le_bytes(data.get(8..10)?.try_into().ok()?) & 0x3fff;
    (kind == b"VP8 "
        && u32::from(vp8_width) == size_24(&frame[6..9])
        && u32::from(vp8_height) == size_24(&frame[9..12]))
    .then_some(())
}

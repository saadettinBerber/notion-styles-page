#[cfg(unix)]
use base64::{Engine, engine::general_purpose::STANDARD};
use notion_sayfa::{ImageInfo, MediaFolder, inspect_image};
#[cfg(unix)]
use notion_sayfa::{ReaderOptions, prepare_document};
use std::fs;

const PNG: &[u8] = include_bytes!("fixtures/raster/one.png");
const JPEG: &[u8] = include_bytes!("fixtures/raster/one.jpeg");
const GIF: &[u8] = include_bytes!("fixtures/raster/one.gif");
const WEBP: &[u8] = include_bytes!("fixtures/raster/one.webp");
const APNG: &[u8] = include_bytes!("fixtures/raster/animated.png");
const ANIMATED_GIF: &[u8] = include_bytes!("fixtures/raster/animated.gif");
const ANIMATED_WEBP: &[u8] = include_bytes!("fixtures/raster/animated.webp");
const ANIMATED_ALPHA_WEBP: &[u8] = include_bytes!("fixtures/raster/animated-alpha.webp");
const SINGLE_FRAMES: &[(&str, &[u8])] =
    &[("png", PNG), ("jpeg", JPEG), ("gif", GIF), ("webp", WEBP)];

#[test]
fn reads_the_actual_type_and_encoded_canvas() {
    for &(kind, bytes) in SINGLE_FRAMES {
        let info = inspect_image(bytes).unwrap();
        assert_eq!(info.media_type, format!("image/{kind}"));
        assert_eq!((info.width, info.height), (1, 1));
    }
    assert_eq!(
        inspect_image(APNG).unwrap(),
        ImageInfo {
            media_type: "image/png",
            width: 3,
            height: 2,
        }
    );
}

#[test]
fn rejects_unsupported_and_incomplete_images() {
    for bytes in [b"".as_slice(), b"<svg/>", b"P6\n1 1\n255\n\xff\0\0"] {
        assert!(inspect_image(bytes).is_err());
    }
    for &(kind, bytes) in SINGLE_FRAMES {
        for missing in 1..=4 {
            assert!(
                inspect_image(&bytes[..bytes.len() - missing]).is_err(),
                "accepted {kind} with {missing} missing bytes"
            );
        }
    }
}

#[test]
fn decodes_every_animation_frame() {
    for (original, second_frame) in [
        (APNG, b"fdAT".as_slice()),
        (ANIMATED_GIF, b"\0\0\xff\0\0\0\0\0\0\0\0\0\x08"),
        (ANIMATED_WEBP, b"\x2f\x02\x40\0\0\x07\x10\xd1"),
    ] {
        let info = inspect_image(original).unwrap();
        assert_eq!((info.width, info.height), (3, 2));
        let mut corrupt = original.to_vec();
        let offset = original
            .windows(second_frame.len())
            .position(|part| part == second_frame)
            .unwrap();
        let code = match info.media_type {
            "image/png" => offset + second_frame.len() + 4,
            "image/gif" => offset + second_frame.len() - 1,
            _ => offset,
        };
        corrupt[code] = 0;
        assert!(inspect_image(&corrupt).is_err(), "{}", info.media_type);
    }
}

#[test]
fn rejects_apng_when_declared_and_physical_frame_counts_differ() {
    for count in [0_u32, 1, 3] {
        let mut bytes = APNG.to_vec();
        rewrite_png_chunk(&mut bytes, b"acTL", |data| {
            data[..4].copy_from_slice(&count.to_be_bytes());
        });
        assert!(inspect_image(&bytes).is_err(), "declared {count} frames");
    }
}

#[test]
fn rejects_png_checksums_even_when_pixels_would_decode() {
    let mut bad_adler = PNG.to_vec();
    rewrite_png_chunk(&mut bad_adler, b"IDAT", |data| {
        *data.last_mut().unwrap() ^= 1;
    });
    assert!(inspect_image(&bad_adler).is_err());

    // The tEXt chunk has a real payload and an intentionally zeroed CRC.
    let mut bad_ancillary_crc = PNG.to_vec();
    let iend = bad_ancillary_crc.len() - 12;
    let text_chunk = b"\0\0\0\x03tEXtk\0v\0\0\0\0";
    bad_ancillary_crc.splice(iend..iend, text_chunk.iter().copied());
    assert!(inspect_image(&bad_ancillary_crc).is_err());
}

fn rewrite_png_chunk(bytes: &mut [u8], kind: &[u8; 4], edit: impl FnOnce(&mut [u8])) {
    let kind_offset = bytes.windows(4).position(|part| part == kind).unwrap();
    let data_start = kind_offset + 4;
    let size = u32::from_be_bytes(bytes[kind_offset - 4..kind_offset].try_into().unwrap());
    let crc_offset = data_start + size as usize;
    edit(&mut bytes[data_start..crc_offset]);
    let crc = crc32fast::hash(&bytes[kind_offset..crc_offset]);
    bytes[crc_offset..crc_offset + 4].copy_from_slice(&crc.to_be_bytes());
}

#[test]
fn rejects_inconsistent_webp_frame_geometry_without_panicking() {
    let info = inspect_image(ANIMATED_ALPHA_WEBP).unwrap();
    assert_eq!((info.width, info.height), (5, 4));

    let mut bytes = ANIMATED_ALPHA_WEBP.to_vec();
    let vp8 = bytes.windows(4).rposition(|part| part == b"VP8 ").unwrap();
    let width_offset = vp8 + 8 + 6;
    bytes[width_offset..width_offset + 2].copy_from_slice(&6_u16.to_le_bytes());
    assert!(inspect_image(&bytes).is_err());
}

#[test]
fn rejects_a_canvas_beyond_the_previous_pixel_limit() {
    let mut bytes = GIF.to_vec();
    bytes[6..8].copy_from_slice(&10_000_u16.to_le_bytes());
    bytes[8..10].copy_from_slice(&10_000_u16.to_le_bytes());
    assert!(inspect_image(&bytes).is_err());
}

#[test]
fn video_and_poster_keep_their_extension_based_mime_and_original_bytes() {
    let directory = tempfile::tempdir().unwrap();
    let media = MediaFolder::new(directory.path().into());
    for path in ["record.mp4", "record.webm", "cover.jpg"] {
        fs::write(directory.path().join(path), b"fixture").unwrap();
    }
    assert_eq!(
        media.video_data_uri("record.mp4").unwrap(),
        "data:video/mp4;base64,Zml4dHVyZQ=="
    );
    assert_eq!(
        media.video_data_uri("record.webm").unwrap(),
        "data:video/webm;base64,Zml4dHVyZQ=="
    );
    assert_eq!(
        media.image_data_uri("cover.jpg").unwrap(),
        "data:image/jpeg;base64,Zml4dHVyZQ=="
    );
}

#[test]
fn video_and_poster_errors_include_the_rejected_path_or_kind() {
    let directory = tempfile::tempdir().unwrap();
    let media = MediaFolder::new(directory.path().into());
    for (error, clue) in [
        (media.video_data_uri("missing.mp4"), "missing.mp4"),
        (media.image_data_uri("record.mp4"), "görsel"),
        (media.video_data_uri("cover.jpg"), "video"),
    ] {
        assert!(error.unwrap_err().to_string().contains(clue));
    }
}

#[cfg(unix)]
#[test]
fn embeds_only_complete_rasters_regardless_of_filename() {
    let directory = tempfile::tempdir().unwrap();
    let media = MediaFolder::new(directory.path().into());
    fs::create_dir(directory.path().join("nested")).unwrap();
    let path = "nested/wrong.txt";
    for &(kind, bytes) in SINGLE_FRAMES {
        fs::write(directory.path().join(path), bytes).unwrap();
        let expected = format!("data:image/{kind};base64,{}", STANDARD.encode(bytes));
        assert_eq!(media.embedded_image_data_uri(path).unwrap(), expected);
        fs::write(directory.path().join(path), &bytes[..bytes.len() / 2]).unwrap();
        let error = media.embedded_image_data_uri(path).unwrap_err().to_string();
        assert!(error.contains(path));
    }
}

#[cfg(unix)]
#[test]
fn renders_resolved_unicode_spaces_and_literal_percent_paths() {
    let directory = tempfile::tempdir().unwrap();
    let media = MediaFolder::new(directory.path().into());
    for (file, target, uri) in [
        ("ş.png", "ş.png", "%C5%9F.png"),
        ("a b.png", "<a b.png>", "a%20b.png"),
        ("a%20b.png", "a%2520b.png", "a%2520b.png"),
    ] {
        fs::write(directory.path().join(file), PNG).unwrap();
        let source = format!("![a]({target})\n");
        let document = prepare_document(&source, ReaderOptions::default()).unwrap();
        let html = document.render(&media).unwrap().html;
        assert!(html.contains("src=\"data:image/png;base64,"));
        assert!(html.contains(&format!("data-source=\"{uri}\"")));
    }
}

#[cfg(unix)]
#[test]
fn rejects_paths_that_escape_the_document_root() {
    let directory = tempfile::tempdir().unwrap();
    let media = MediaFolder::new(directory.path().into());
    fs::create_dir(directory.path().join("nested")).unwrap();
    fs::write(directory.path().join("nested/one.png"), PNG).unwrap();
    let absolute = directory.path().join("nested/one.png");
    for path in [
        absolute.to_str().unwrap(),
        "../one.png",
        "missing.png",
        ".",
        "nested",
        "nested/../../one.png",
        "%2e%2e/one.png",
        "nested/%2e%2e/nested/one.png",
        "nested%2Fone.png",
        "nested%5cone.png",
    ] {
        assert!(media.embedded_image_data_uri(path).is_err(), "{path}");
    }
}

#[cfg(unix)]
#[test]
fn rejects_symlinks_and_named_pipes() {
    use std::os::unix::fs::symlink;
    use std::process::Command;

    let directory = tempfile::tempdir().unwrap();
    let media = MediaFolder::new(directory.path().into());
    let root = directory.path();
    fs::create_dir(root.join("real")).unwrap();
    fs::write(root.join("real/one.png"), PNG).unwrap();
    symlink("real/one.png", root.join("file.png")).unwrap();
    symlink("real", root.join("linked")).unwrap();
    assert!(
        Command::new("mkfifo")
            .arg(root.join("pipe.png"))
            .status()
            .unwrap()
            .success()
    );
    for path in ["file.png", "linked/one.png", "pipe.png"] {
        assert!(media.embedded_image_data_uri(path).is_err(), "{path}");
    }
}

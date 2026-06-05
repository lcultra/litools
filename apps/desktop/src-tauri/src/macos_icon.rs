use std::{ffi::c_void, path::Path};

use image::{DynamicImage, ImageFormat};
use objc2::msg_send;
use objc2_app_kit::NSWorkspace;
use objc2_foundation::{NSData, NSString};

pub fn app_icon_png(path: &Path) -> std::io::Result<Vec<u8>> {
    let tiff = app_icon_tiff(path)?;
    let image = image::load_from_memory_with_format(&tiff, ImageFormat::Tiff)
        .map_err(std::io::Error::other)?;
    dynamic_image_to_png(image)
}

fn app_icon_tiff(path: &Path) -> std::io::Result<Vec<u8>> {
    let path = path
        .to_str()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid app path"))?;
    let path = NSString::from_str(path);

    let workspace = NSWorkspace::sharedWorkspace();
    let image = workspace.iconForFile(&path);
    let data = image
        .TIFFRepresentation()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "missing icon tiff"))?;

    Ok(nsdata_to_vec(&data))
}

fn nsdata_to_vec(data: &NSData) -> Vec<u8> {
    let length = data.length() as usize;
    let bytes = unsafe {
        let bytes: *const c_void = msg_send![data, bytes];
        bytes.cast::<u8>()
    };

    unsafe { std::slice::from_raw_parts(bytes, length).to_vec() }
}

fn dynamic_image_to_png(image: DynamicImage) -> std::io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut bytes);
    image
        .write_to(&mut cursor, ImageFormat::Png)
        .map_err(std::io::Error::other)?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_calendar_icon_from_system_workspace_when_available() {
        let path = Path::new("/System/Applications/Calendar.app");
        if !path.exists() {
            return;
        }

        let bytes = app_icon_png(path).expect("calendar icon png");

        assert!(bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
    }
}

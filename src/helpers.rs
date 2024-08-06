use std::{collections::HashSet, io::Write, path::PathBuf};

use crate::{
    common::{BitMatrix, HybridBinarizer, Result},
    multi::{GenericMultipleBarcodeReader, MultipleBarcodeReader},
    BarcodeFormat, BinaryBitmap, DecodeHints, Exceptions, FilteredImageReader,
    Luma8LuminanceSource, MultiFormatReader, MultiUseMultiFormatReader, RXingResult, Reader,
};

#[cfg(feature = "image")]
use crate::BufferedImageLuminanceSource;

#[cfg(feature = "svg_read")]
pub fn detect_in_svg(file_name: &str, barcode_type: Option<BarcodeFormat>) -> Result<RXingResult> {
    detect_in_svg_with_hints(file_name, barcode_type, &mut DecodeHints::default())
}

#[cfg(feature = "svg_read")]
pub fn detect_in_svg_with_hints(
    file_name: &str,
    barcode_type: Option<BarcodeFormat>,
    hints: &DecodeHints,
) -> Result<RXingResult> {
    use std::{fs::File, io::Read};

    use crate::SVGLuminanceSource;

    let mut hints = hints.clone();

    let path = PathBuf::from(file_name);
    if !path.exists() {
        return Err(Exceptions::illegal_argument_with("file does not exist"));
    }

    let Ok(mut file) = File::open(path) else {
        return Err(Exceptions::illegal_argument_with("file cannot be opened"));
    };

    let mut svg_data = Vec::new();
    if file.read_to_end(&mut svg_data).is_err() {
        return Err(Exceptions::illegal_argument_with("file cannot be read"));
    }

    let mut multi_format_reader = MultiFormatReader::default();

    if let Some(bc_type) = barcode_type {
        hints.PossibleFormats = Some(HashSet::from([bc_type]));
    }

    if hints.TryHarder.is_none() {
        hints.TryHarder = Some(true);
    }

    multi_format_reader.decode_with_hints(
        &mut BinaryBitmap::new(HybridBinarizer::new(SVGLuminanceSource::new(&svg_data)?)),
        &hints,
    )
}

#[cfg(feature = "svg_read")]
pub fn detect_multiple_in_svg(file_name: &str) -> Result<Vec<RXingResult>> {
    detect_multiple_in_svg_with_hints(file_name, &mut DecodeHints::default())
}

#[cfg(feature = "svg_read")]
pub fn detect_multiple_in_svg_with_hints(
    file_name: &str,
    hints: &DecodeHints,
) -> Result<Vec<RXingResult>> {
    use std::{fs::File, io::Read};

    use crate::SVGLuminanceSource;

    let mut hints = hints.clone();

    let path = PathBuf::from(file_name);
    if !path.exists() {
        return Err(Exceptions::illegal_argument_with("file does not exist"));
    }

    let Ok(mut file) = File::open(path) else {
        return Err(Exceptions::illegal_argument_with("file cannot be opened"));
    };

    let mut svg_data = Vec::new();
    if file.read_to_end(&mut svg_data).is_err() {
        return Err(Exceptions::illegal_argument_with("file cannot be read"));
    }

    let multi_format_reader = MultiUseMultiFormatReader::default();
    let mut scanner = GenericMultipleBarcodeReader::new(multi_format_reader);

    if hints.TryHarder.is_none() {
        hints.TryHarder = Some(true);
    }

    scanner.decode_multiple_with_hints(
        &mut BinaryBitmap::new(HybridBinarizer::new(SVGLuminanceSource::new(&svg_data)?)),
        &hints,
    )
}

#[cfg(feature = "image")]
pub fn detect_in_file(file_name: &str, barcode_type: Option<BarcodeFormat>) -> Result<RXingResult> {
    detect_in_file_with_hints(file_name, barcode_type, &mut DecodeHints::default())
}

#[cfg(feature = "image")]
pub fn detect_in_file_with_hints(
    file_name: &str,
    barcode_type: Option<BarcodeFormat>,
    hints: &DecodeHints,
) -> Result<RXingResult> {
    let Ok(img) = image::open(file_name) else {
        return Err(Exceptions::illegal_argument_with(format!(
            "file '{file_name}' not found or cannot be opened"
        )));
    };
    let mut multi_format_reader = MultiFormatReader::default();
    let mut hints = hints.clone();

    if let Some(bc_type) = barcode_type {
        hints.PossibleFormats = Some(HashSet::from([bc_type]));
    }

    if hints.TryHarder.is_none() {
        hints.TryHarder = Some(true);
    }

    multi_format_reader.decode_with_hints(
        &mut BinaryBitmap::new(HybridBinarizer::new(BufferedImageLuminanceSource::new(img))),
        &hints,
    )
}

#[cfg(feature = "image")]
pub fn detect_multiple_in_file(file_name: &str) -> Result<Vec<RXingResult>> {
    detect_multiple_in_file_with_hints(file_name, &mut DecodeHints::default())
}

#[cfg(feature = "image")]
pub fn detect_multiple_in_file_with_hints(
    file_name: &str,
    hints: &DecodeHints,
) -> Result<Vec<RXingResult>> {
    let img = image::open(file_name)
        .map_err(|e| Exceptions::runtime_with(format!("couldn't read {file_name}: {e}")))?;
    let multi_format_reader = MultiUseMultiFormatReader::default();
    let mut scanner = GenericMultipleBarcodeReader::new(multi_format_reader);
    let mut hints = hints.clone();

    if hints.TryHarder.is_none() {
        hints.TryHarder = Some(true);
    }

    scanner.decode_multiple_with_hints(
        &mut BinaryBitmap::new(HybridBinarizer::new(BufferedImageLuminanceSource::new(img))),
        &hints,
    )
}

pub fn detect_in_luma(
    luma: Vec<u8>,
    width: u32,
    height: u32,
    barcode_type: Option<BarcodeFormat>,
) -> Result<RXingResult> {
    detect_in_luma_with_hints(
        luma,
        height,
        width,
        barcode_type,
        &mut DecodeHints::default(),
    )
}

pub fn detect_in_luma_with_hints(
    luma: Vec<u8>,
    width: u32,
    height: u32,
    barcode_type: Option<BarcodeFormat>,
    hints: &DecodeHints,
) -> Result<RXingResult> {
    let mut multi_format_reader = MultiFormatReader::default();
    let mut hints = hints.clone();

    if let Some(bc_type) = barcode_type {
        hints.PossibleFormats = Some(HashSet::from([bc_type]));
    }

    if hints.TryHarder.is_none() {
        hints.TryHarder = Some(true);
    }

    multi_format_reader.decode_with_hints(
        &mut BinaryBitmap::new(HybridBinarizer::new(Luma8LuminanceSource::new(
            luma, width, height,
        ))),
        &hints,
    )
}

pub fn detect_in_luma_filtered(
    luma: Vec<u8>,
    width: u32,
    height: u32,
    barcode_type: Option<BarcodeFormat>,
) -> Result<RXingResult> {
    crate::helpers::detect_in_luma_filtered_with_hints(
        luma,
        height,
        width,
        barcode_type,
        &mut DecodeHints::default(),
    )
}

pub fn detect_in_luma_filtered_with_hints(
    luma: Vec<u8>,
    width: u32,
    height: u32,
    barcode_type: Option<BarcodeFormat>,
    hints: &DecodeHints,
) -> Result<RXingResult> {
    let mut multi_format_reader = FilteredImageReader::new(MultiFormatReader::default());
    let mut hints = hints.clone();

    if let Some(bc_type) = barcode_type {
        hints.PossibleFormats = Some(HashSet::from([bc_type]));
    }

    if hints.TryHarder.is_none() {
        hints.TryHarder = Some(true);
    }

    multi_format_reader.decode_with_hints(
        &mut BinaryBitmap::new(HybridBinarizer::new(Luma8LuminanceSource::new(
            luma, width, height,
        ))),
        &hints,
    )
}

pub fn detect_multiple_in_luma(luma: Vec<u8>, width: u32, height: u32) -> Result<Vec<RXingResult>> {
    detect_multiple_in_luma_with_hints(luma, width, height, &mut DecodeHints::default())
}

pub fn detect_multiple_in_luma_with_hints(
    luma: Vec<u8>,
    width: u32,
    height: u32,
    hints: &DecodeHints,
) -> Result<Vec<RXingResult>> {
    let multi_format_reader = MultiUseMultiFormatReader::default();
    let mut scanner = GenericMultipleBarcodeReader::new(multi_format_reader);
    let mut hints = hints.clone();

    if hints.TryHarder.is_none() {
        hints.TryHarder = Some(true);
    }

    scanner.decode_multiple_with_hints(
        &mut BinaryBitmap::new(HybridBinarizer::new(Luma8LuminanceSource::new(
            luma, width, height,
        ))),
        &hints,
    )
}

#[cfg(feature = "image")]
pub fn save_image(file_name: &str, bit_matrix: &BitMatrix) -> Result<()> {
    let image: image::DynamicImage = bit_matrix.into();
    match image.save(file_name) {
        Ok(_) => Ok(()),
        Err(err) => Err(Exceptions::illegal_argument_with(format!(
            "could not save file '{file_name}': {err}"
        ))),
    }
}

#[cfg(feature = "svg_write")]
pub fn save_svg(file_name: &str, bit_matrix: &BitMatrix) -> Result<()> {
    let svg: svg::Document = bit_matrix.into();

    match svg::save(file_name, &svg) {
        Ok(_) => Ok(()),
        Err(err) => Err(Exceptions::illegal_argument_with(format!(
            "could not save file '{}': {}",
            file_name, err
        ))),
    }
}

pub fn save_file(file_name: &str, bit_matrix: &BitMatrix) -> Result<()> {
    let path = PathBuf::from(file_name);

    #[allow(unused_variables)]
    let ext: String = if let Some(e) = path.extension() {
        e.to_string_lossy().to_string()
    } else {
        String::default()
    };

    #[cfg(feature = "svg_write")]
    if ext == "svg" {
        return save_svg(file_name, bit_matrix);
    }

    #[cfg(feature = "image")]
    if !ext.is_empty() && ext != "txt" {
        return save_image(file_name, bit_matrix);
    }

    let result_tester = || -> std::io::Result<_> {
        let file = std::fs::File::create(path)?;
        let mut output = std::io::BufWriter::new(file);
        output.write_all(bit_matrix.to_string().as_bytes())?;
        output.flush()?;
        Ok(())
    };

    match result_tester() {
        Ok(_) => Ok(()),
        Err(_) => Err(Exceptions::illegal_argument_with(format!(
            "could not write to '{file_name}'"
        ))),
    }
}

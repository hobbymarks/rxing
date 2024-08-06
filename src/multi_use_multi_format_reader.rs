/*
 * Copyright 2007 ZXing authors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::collections::HashSet;

use crate::common::Result;
use crate::qrcode::cpp_port::QrReader;
use crate::DecodeHints;
use crate::{
    aztec::AztecReader, datamatrix::DataMatrixReader, maxicode::MaxiCodeReader,
    oned::MultiFormatOneDReader, pdf417::PDF417Reader, qrcode::QRCodeReader, BarcodeFormat,
    Binarizer, BinaryBitmap, Exceptions, RXingResult, Reader,
};

pub(crate) const ONE_D_FORMATS: [BarcodeFormat; 12] = [
    BarcodeFormat::UPC_A,
    BarcodeFormat::UPC_E,
    BarcodeFormat::EAN_13,
    BarcodeFormat::EAN_8,
    BarcodeFormat::CODABAR,
    BarcodeFormat::CODE_39,
    BarcodeFormat::CODE_93,
    BarcodeFormat::CODE_128,
    BarcodeFormat::ITF,
    BarcodeFormat::RSS_14,
    BarcodeFormat::RSS_EXPANDED,
    BarcodeFormat::TELEPEN,
];

/**
 * MultiFormatReader is a convenience class and the main entry point into the library for most uses.
 * By default it attempts to decode all barcode formats that the library supports. Optionally, you
 * can provide a hints object to request different behavior, for example only decoding QR codes.
 *
 * @author Sean Owen
 * @author dswitkin@google.com (Daniel Switkin)
 */
#[derive(Default)]
pub struct MultiUseMultiFormatReader {
    hints: DecodeHints,
    possible_formats: HashSet<BarcodeFormat>,
    try_harder: bool,
    one_d_reader: MultiFormatOneDReader,
    qr_code_reader: QRCodeReader,
    data_matrix_reader: DataMatrixReader,
    aztec_reader: AztecReader,
    pdf417_reader: PDF417Reader,
    maxicode_reader: MaxiCodeReader,
    cpp_qrcode_reader: QrReader,
}

impl Reader for MultiUseMultiFormatReader {
    /**
     * This version of decode honors the intent of Reader.decode(BinaryBitmap) in that it
     * passes null as a hint to the decoders. However, that makes it inefficient to call repeatedly.
     * Use setHints() followed by decodeWithState() for continuous scan applications.
     *
     * @param image The pixel data to decode
     * @return The contents of the image
     * @throws NotFoundException Any errors which occurred
     */
    fn decode<B: Binarizer>(&mut self, image: &mut BinaryBitmap<B>) -> Result<RXingResult> {
        self.set_hints(&DecodeHints::default());
        self.decode_internal(image)
    }

    /**
     * Decode an image using the hints provided. Does not honor existing state.
     *
     * @param image The pixel data to decode
     * @param hints The hints to use, clearing the previous state.
     * @return The contents of the image
     * @throws NotFoundException Any errors which occurred
     */
    fn decode_with_hints<B: Binarizer>(
        &mut self,
        image: &mut BinaryBitmap<B>,
        hints: &DecodeHints,
    ) -> Result<RXingResult> {
        self.set_hints(hints);
        self.decode_internal(image)
    }

    fn reset(&mut self) {
        self.one_d_reader.reset();
        self.qr_code_reader.reset();
        self.data_matrix_reader.reset();
        self.aztec_reader.reset();
        self.pdf417_reader.reset();
        self.maxicode_reader.reset();
        self.cpp_qrcode_reader.reset();
    }
}

impl MultiUseMultiFormatReader {
    /**
     * Decode an image using the state set up by calling setHints() previously. Continuous scan
     * clients will get a <b>large</b> speed increase by using this instead of decode().
     *
     * @param image The pixel data to decode
     * @return The contents of the image
     * @throws NotFoundException Any errors which occurred
     */
    pub fn decode_with_state<B: Binarizer>(
        &mut self,
        image: &mut BinaryBitmap<B>,
    ) -> Result<RXingResult> {
        // Make sure to set up the default state so we don't crash
        if self.possible_formats.is_empty() {
            self.set_hints(&DecodeHints::default());
        }
        self.decode_internal(image)
    }

    /**
     * This method adds state to the MultiFormatReader. By setting the hints once, subsequent calls
     * to decodeWithState(image) can reuse the same set of readers without reallocating memory. This
     * is important for performance in continuous scan clients.
     *
     * @param hints The set of hints to use for subsequent calls to decode(image)
     */
    pub fn set_hints(&mut self, hints: &DecodeHints) {
        self.hints.clone_from(hints);

        self.try_harder = self.hints.TryHarder.unwrap_or(false);
        self.possible_formats = if let Some(formats) = &hints.PossibleFormats {
            formats.clone()
        } else {
            HashSet::new()
        };
        self.one_d_reader = MultiFormatOneDReader::new(hints);
    }

    pub fn decode_internal<B: Binarizer>(
        &mut self,
        image: &mut BinaryBitmap<B>,
    ) -> Result<RXingResult> {
        let res = self.decode_formats(image);
        if res.is_ok() {
            return res;
        }
        if self.hints.AlsoInverted.unwrap_or(false) {
            // Calling all readers again with inverted image
            image.get_black_matrix_mut().flip_self();
            let res = self.decode_formats(image);
            if res.is_ok() {
                let mut r = res.unwrap();
                r.putMetadata(
                    crate::RXingResultMetadataType::IS_INVERTED,
                    crate::RXingResultMetadataValue::IsInverted(true),
                );
                return Ok(r);
            }
            // if res.is_ok() {
            //     return res;
            // }
        }
        Err(Exceptions::NOT_FOUND)
    }

    fn decode_formats<B: Binarizer>(&mut self, image: &mut BinaryBitmap<B>) -> Result<RXingResult> {
        if !self.possible_formats.is_empty() {
            let one_d = ONE_D_FORMATS
                .iter()
                .any(|e| self.possible_formats.contains(e));
            // let one_d = self.possible_formats.contains(&BarcodeFormat::UPC_A)
            //     || self.possible_formats.contains(&BarcodeFormat::UPC_E)
            //     || self.possible_formats.contains()
            //     || self.possible_formats.contains()
            //     || self.possible_formats.contains()
            //     || self.possible_formats.contains()
            //     || self.possible_formats.contains()
            //     || self.possible_formats.contains()
            //     || self.possible_formats.contains()
            //     || self.possible_formats.contains()
            //     || self.possible_formats.contains()
            //     || self.possible_formats.contains();
            if one_d && !self.try_harder {
                if let Ok(res) = self.one_d_reader.decode_with_hints(image, &self.hints) {
                    return Ok(res);
                }
            }
            for possible_format in self.possible_formats.iter() {
                let res = match possible_format {
                    BarcodeFormat::QR_CODE => {
                        let a = self.cpp_qrcode_reader.decode_with_hints(image, &self.hints);
                        if a.is_ok() {
                            a
                        } else {
                            self.qr_code_reader.decode_with_hints(image, &self.hints)
                        }
                    }
                    BarcodeFormat::MICRO_QR_CODE => {
                        self.cpp_qrcode_reader.decode_with_hints(image, &self.hints)
                    }
                    BarcodeFormat::DATA_MATRIX => self
                        .data_matrix_reader
                        .decode_with_hints(image, &self.hints),
                    BarcodeFormat::AZTEC => self.aztec_reader.decode_with_hints(image, &self.hints),
                    BarcodeFormat::PDF_417 => {
                        self.pdf417_reader.decode_with_hints(image, &self.hints)
                    }
                    BarcodeFormat::MAXICODE => {
                        self.maxicode_reader.decode_with_hints(image, &self.hints)
                    }
                    _ => Err(Exceptions::UNSUPPORTED_OPERATION),
                };
                if res.is_ok() {
                    return res;
                }
            }
            if one_d && self.try_harder {
                if let Ok(res) = self.one_d_reader.decode_with_hints(image, &self.hints) {
                    return Ok(res);
                }
            }
        } else {
            if !self.try_harder {
                if let Ok(res) = self.one_d_reader.decode_with_hints(image, &self.hints) {
                    return Ok(res);
                }
            }
            if let Ok(res) = self.cpp_qrcode_reader.decode_with_hints(image, &self.hints) {
                return Ok(res);
            }
            if let Ok(res) = self.qr_code_reader.decode_with_hints(image, &self.hints) {
                return Ok(res);
            }
            if let Ok(res) = self
                .data_matrix_reader
                .decode_with_hints(image, &self.hints)
            {
                return Ok(res);
            }
            if let Ok(res) = self.aztec_reader.decode_with_hints(image, &self.hints) {
                return Ok(res);
            }
            if let Ok(res) = self.pdf417_reader.decode_with_hints(image, &self.hints) {
                return Ok(res);
            }
            if let Ok(res) = self.maxicode_reader.decode_with_hints(image, &self.hints) {
                return Ok(res);
            }

            if self.try_harder {
                if let Ok(res) = self.one_d_reader.decode_with_hints(image, &self.hints) {
                    return Ok(res);
                }
            }
        }

        Err(Exceptions::UNSUPPORTED_OPERATION)
    }
}

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

use std::{collections::HashMap, marker::PhantomData};

use crate::{
    common::{BitMatrix, DetectorRXingResult},
    BarcodeFormat, DecodeHintType, Exceptions, RXingResult, RXingResultMetadataType,
    RXingResultMetadataValue, Reader, LuminanceSource, Binarizer,
};

use super::{decoder::Decoder, detector::Detector};

use lazy_static::lazy_static;

lazy_static! {
    static ref DECODER: Decoder = Decoder::new();
}

/**
 * This implementation can detect and decode Data Matrix codes in an image.
 *
 * @author bbrown@google.com (Brian Brown)
 */
pub struct DataMatrixReader<L:LuminanceSource,B:Binarizer<L>>{pd_l: PhantomData<L>, pd_b: PhantomData<B>}

// private static final RXingResultPoint[] NO_POINTS = new RXingResultPoint[0];

// private final Decoder decoder = new Decoder();

impl<L:LuminanceSource,B:Binarizer<L>> Reader<L,B> for DataMatrixReader<L,B> {
    /**
     * Locates and decodes a Data Matrix code in an image.
     *
     * @return a String representing the content encoded by the Data Matrix code
     * @throws NotFoundException if a Data Matrix code cannot be found
     * @throws FormatException if a Data Matrix code cannot be decoded
     * @throws ChecksumException if error correction fails
     */
    fn decode(
        &mut self,
        image: &crate::BinaryBitmap<L,B>,
    ) -> Result<crate::RXingResult, crate::Exceptions> {
        self.decode_with_hints(image, &HashMap::new())
    }

    /**
     * Locates and decodes a Data Matrix code in an image.
     *
     * @return a String representing the content encoded by the Data Matrix code
     * @throws NotFoundException if a Data Matrix code cannot be found
     * @throws FormatException if a Data Matrix code cannot be decoded
     * @throws ChecksumException if error correction fails
     */
    fn decode_with_hints(
        &mut self,
        image: &crate::BinaryBitmap<L,B>,
        hints: &crate::DecodingHintDictionary,
    ) -> Result<crate::RXingResult, crate::Exceptions> {
        let decoderRXingResult;
        let mut points = Vec::new();
        if hints.contains_key(&DecodeHintType::PURE_BARCODE) {
            let bits = self.extractPureBits(image.getBlackMatrix())?;
            decoderRXingResult = DECODER.decode(&bits)?;
            points.clear();
        } else {
            let detectorRXingResult = Detector::new(image.getBlackMatrix().clone())?.detect()?;
            decoderRXingResult = DECODER.decode(detectorRXingResult.getBits())?;
            points = detectorRXingResult.getPoints().clone();
        }
        let mut result = RXingResult::new(
            decoderRXingResult.getText().clone(),
            decoderRXingResult.getRawBytes().clone(),
            points.clone(),
            BarcodeFormat::DATA_MATRIX,
        );
        let byteSegments = decoderRXingResult.getByteSegments();
        if !byteSegments.is_empty() {
            result.putMetadata(
                RXingResultMetadataType::BYTE_SEGMENTS,
                RXingResultMetadataValue::ByteSegments(byteSegments.clone()),
            );
        }
        let ecLevel = decoderRXingResult.getECLevel();
        if !ecLevel.is_empty() {
            result.putMetadata(
                RXingResultMetadataType::ERROR_CORRECTION_LEVEL,
                RXingResultMetadataValue::ErrorCorrectionLevel(ecLevel.to_string()),
            );
        }
        result.putMetadata(
            RXingResultMetadataType::SYMBOLOGY_IDENTIFIER,
            RXingResultMetadataValue::SymbologyIdentifier(format!(
                "]d{}",
                decoderRXingResult.getSymbologyModifier()
            )),
        );

        Ok(result)
    }

    fn reset(&mut self) {
        // do nothing
    }
}

impl<L:LuminanceSource,B:Binarizer<L>> DataMatrixReader<L,B> {
    /**
     * This method detects a code in a "pure" image -- that is, pure monochrome image
     * which contains only an unrotated, unskewed, image of a code, with some white border
     * around it. This is a specialized method that works exceptionally fast in this special
     * case.
     */
    fn extractPureBits(&self, image: &BitMatrix) -> Result<BitMatrix, Exceptions> {
        let Some(leftTopBlack) = image.getTopLeftOnBit() else {
      return Err(Exceptions::NotFoundException("".to_owned()))
    };
        let Some(rightBottomBlack) = image.getBottomRightOnBit()else {
      return Err(Exceptions::NotFoundException("".to_owned()))
    };

        let moduleSize = Self::moduleSize(&leftTopBlack, image)?;

        let mut top = leftTopBlack[1];
        let bottom = rightBottomBlack[1];
        let mut left = leftTopBlack[0];
        let right = rightBottomBlack[0];

        let matrixWidth = (right as i32 - left as i32 + 1) / moduleSize as i32;
        let matrixHeight = (bottom as i32 - top as i32 + 1) / moduleSize as i32;
        if matrixWidth <= 0 || matrixHeight <= 0 {
            return Err(Exceptions::NotFoundException("".to_owned()));
            // throw NotFoundException.getNotFoundInstance();
        }

        let matrixWidth = matrixWidth as u32;
        let matrixHeight = matrixHeight as u32;

        // Push in the "border" by half the module width so that we start
        // sampling in the middle of the module. Just in case the image is a
        // little off, this will help recover.
        let nudge = moduleSize / 2;
        top += nudge;
        left += nudge;

        // Now just read off the bits
        let mut bits = BitMatrix::new(matrixWidth, matrixHeight)?;
        for y in 0..matrixHeight {
            // for (int y = 0; y < matrixHeight; y++) {
            let iOffset = top + y * moduleSize;
            for x in 0..matrixWidth {
                // for (int x = 0; x < matrixWidth; x++) {
                if image.get(left + x * moduleSize, iOffset) {
                    bits.set(x, y);
                }
            }
        }
        Ok(bits)
    }

    fn moduleSize(leftTopBlack: &[u32], image: &BitMatrix) -> Result<u32, Exceptions> {
        let width = image.getWidth();
        let mut x = leftTopBlack[0];
        let y = leftTopBlack[1];
        while x < width && image.get(x, y) {
            x += 1;
        }
        if x == width {
            return Err(Exceptions::NotFoundException("".to_owned()));
        }

        let moduleSize = x - leftTopBlack[0];
        if moduleSize == 0 {
            return Err(Exceptions::NotFoundException("".to_owned()));
        }

        Ok(moduleSize)
    }
}

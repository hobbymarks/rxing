/*
 * Copyright 2011 ZXing authors
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
    common::BitMatrix, BarcodeFormat, Exceptions, RXingResult, RXingResultMetadataType, Reader, LuminanceSource, Binarizer,
};

use super::decoder::decoder;

/**
 * This implementation can detect and decode a MaxiCode in an image.
 */
pub struct MaxiCodeReader<L:LuminanceSource,B:Binarizer<L>> {
    // private final Decoder decoder = new Decoder();
    pd_l: PhantomData<L>,
    pd_b: PhantomData<B>
}

impl<L:LuminanceSource,B:Binarizer<L>> Reader<L,B> for MaxiCodeReader<L,B> {
    /**
     * Locates and decodes a MaxiCode in an image.
     *
     * @return a String representing the content encoded by the MaxiCode
     * @throws NotFoundException if a MaxiCode cannot be found
     * @throws FormatException if a MaxiCode cannot be decoded
     * @throws ChecksumException if error correction fails
     */
    fn decode(
        &mut self,
        image: &crate::BinaryBitmap<L,B>,
    ) -> Result<crate::RXingResult, crate::Exceptions> {
        self.decode_with_hints(image, &HashMap::new())
    }

    /**
     * Locates and decodes a MaxiCode in an image.
     *
     * @return a String representing the content encoded by the MaxiCode
     * @throws NotFoundException if a MaxiCode cannot be found
     * @throws FormatException if a MaxiCode cannot be decoded
     * @throws ChecksumException if error correction fails
     */
    fn decode_with_hints(
        &mut self,
        image: &crate::BinaryBitmap<L,B>,
        hints: &crate::DecodingHintDictionary,
    ) -> Result<crate::RXingResult, crate::Exceptions> {
        // Note that MaxiCode reader effectively always assumes PURE_BARCODE mode
        // and can't detect it in an image
        let bits = Self::extractPureBits(image.getBlackMatrix())?;
        let decoderRXingResult = decoder::decode_with_hints(bits, &hints)?;
        let mut result = RXingResult::new(
            decoderRXingResult.getText(),
            decoderRXingResult.getRawBytes().clone(),
            Vec::new(),
            BarcodeFormat::MAXICODE,
        );

        let ecLevel = decoderRXingResult.getECLevel();
        if !ecLevel.is_empty() {
            result.putMetadata(
                RXingResultMetadataType::ERROR_CORRECTION_LEVEL,
                crate::RXingResultMetadataValue::ErrorCorrectionLevel(ecLevel.to_owned()),
            );
        }
        Ok(result)
    }

    fn reset(&mut self) {
        // do nothing
    }
}
impl<L:LuminanceSource,B:Binarizer<L>> MaxiCodeReader<L,B> {
    const MATRIX_WIDTH: u32 = 30;
    const MATRIX_HEIGHT: u32 = 33;

    /**
     * This method detects a code in a "pure" image -- that is, pure monochrome image
     * which contains only an unrotated, unskewed, image of a code, with some white border
     * around it. This is a specialized method that works exceptionally fast in this special
     * case.
     */
    fn extractPureBits(image: &BitMatrix) -> Result<BitMatrix, Exceptions> {
        let enclosingRectangleOption = image.getEnclosingRectangle();
        if enclosingRectangleOption.is_none() {
            return Err(Exceptions::NotFoundException("".to_owned()));
        }

        let enclosingRectangle = enclosingRectangleOption.unwrap();

        let left = enclosingRectangle[0];
        let top = enclosingRectangle[1];
        let width = enclosingRectangle[2];
        let height = enclosingRectangle[3];

        // Now just read off the bits
        let mut bits = BitMatrix::new(Self::MATRIX_WIDTH, Self::MATRIX_HEIGHT)?;
        for y in 0..Self::MATRIX_HEIGHT {
            // for (int y = 0; y < MATRIX_HEIGHT; y++) {
            let iy = (top + (y * height + height / 2) / Self::MATRIX_HEIGHT).min(height - 1);
            for x in 0..Self::MATRIX_WIDTH {
                // for (int x = 0; x < MATRIX_WIDTH; x++) {
                // srowen: I don't quite understand why the formula below is necessary, but it
                // can walk off the image if left + width = the right boundary. So cap it.
                let ix = left
                    + ((x * width + width / 2 + (y & 0x01) * width / 2) / Self::MATRIX_WIDTH)
                        .min(width - 1);
                if image.get(ix, iy) {
                    bits.set(x, y);
                }
            }
        }
        Ok(bits)
    }
}

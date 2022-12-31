/*
 * Copyright 2009 ZXing authors
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
    multi::MultipleBarcodeReader, BarcodeFormat, BinaryBitmap, DecodingHintDictionary, Exceptions,
    RXingResult, RXingResultMetadataType, RXingResultMetadataValue, RXingResultPoint, Reader,
    ResultPoint, LuminanceSource, Binarizer,
};

use super::{
    decoder::pdf_417_scanning_decoder, detector::detector, pdf_417_common,
    PDF417RXingResultMetadata,
};

/**
 * This implementation can detect and decode PDF417 codes in an image.
 *
 * @author Guenther Grau
 */
pub struct PDF417Reader<L:LuminanceSource,B:Binarizer<L>>(PhantomData<L>,PhantomData<B>);

impl<L:LuminanceSource,B:Binarizer<L>> Reader<L,B> for PDF417Reader<L,B> {
    /**
     * Locates and decodes a PDF417 code in an image.
     *
     * @return a String representing the content encoded by the PDF417 code
     * @throws NotFoundException if a PDF417 code cannot be found,
     * @throws FormatException if a PDF417 cannot be decoded
     */
    fn decode(
        &mut self,
        image: &crate::BinaryBitmap<L,B>,
    ) -> Result<crate::RXingResult, crate::Exceptions> {
        self.decode_with_hints(image, &HashMap::new())
    }

    fn decode_with_hints(
        &mut self,
        image: &crate::BinaryBitmap<L,B>,
        hints: &crate::DecodingHintDictionary,
    ) -> Result<crate::RXingResult, crate::Exceptions> {
        let result = Self::decode(image, hints, false)?;
        if result.is_empty() {
            // if (result.length == 0 || result[0] == null) {
            return Err(Exceptions::NotFoundException("".to_owned()));
        }
        Ok(result[0].clone())
    }
}
impl<L:LuminanceSource,B:Binarizer<L>> MultipleBarcodeReader<L,B> for PDF417Reader<L,B> {
    fn decode_multiple(
        &mut self,
        image: &crate::BinaryBitmap<L,B>,
    ) -> Result<Vec<crate::RXingResult>, crate::Exceptions> {
        self.decode_multiple_with_hints(image, &HashMap::new())
    }

    fn decode_multiple_with_hints(
        &mut self,
        image: &crate::BinaryBitmap<L,B>,
        hints: &crate::DecodingHintDictionary,
    ) -> Result<Vec<crate::RXingResult>, crate::Exceptions> {
        //try {
        Self::decode(image, hints, true)
        //} catch (FormatException | ChecksumException ignored) {
        //  throw NotFoundException.getNotFoundInstance();
        //}
    }
}

impl<L:LuminanceSource,B:Binarizer<L>> Default for PDF417Reader<L,B> {
    fn default() -> Self {
        Self(PhantomData,PhantomData)
    }
}

impl<L:LuminanceSource,B:Binarizer<L>> PDF417Reader<L,B> {
    pub fn new() -> Self {
        Self::default()
    }

    fn decode(
        image: &BinaryBitmap<L,B>,
        hints: &DecodingHintDictionary,
        multiple: bool,
    ) -> Result<Vec<RXingResult>, Exceptions> {
        let mut results = Vec::new(); //new ArrayList<>();
        let detectorRXingResult = detector::detect_with_hints(image, hints, multiple)?;
        for points in detectorRXingResult.getPoints() {
            let points_filtered = points.iter().filter_map(|e| e.clone()).collect();
            // for (RXingResultPoint[] points : detectorRXingResult.getPoints()) {
            let decoderRXingResult = pdf_417_scanning_decoder::decode(
                detectorRXingResult.getBits(),
                points[4],
                points[5],
                points[6],
                points[7],
                Self::getMinCodewordWidth(points),
                Self::getMaxCodewordWidth(points),
            )?;
            let mut result = RXingResult::new(
                decoderRXingResult.getText(),
                decoderRXingResult.getRawBytes().clone(),
                points_filtered,
                BarcodeFormat::PDF_417,
            );

            result.putMetadata(
                RXingResultMetadataType::ERROR_CORRECTION_LEVEL,
                RXingResultMetadataValue::ErrorCorrectionLevel(
                    decoderRXingResult.getECLevel().to_owned(),
                ),
            );

            if let Some(pdf417RXingResultMetadata) = decoderRXingResult.getOther() {
                if pdf417RXingResultMetadata.is::<PDF417RXingResultMetadata>() {
                    let data = RXingResultMetadataValue::Pdf417ExtraMetadata(
                        pdf417RXingResultMetadata
                            .clone()
                            .downcast::<PDF417RXingResultMetadata>()
                            .unwrap(),
                    );
                    result.putMetadata(RXingResultMetadataType::PDF417_EXTRA_METADATA, data);
                }
            }
            // PDF417RXingResultMetadata pdf417RXingResultMetadata = (PDF417RXingResultMetadata) decoderRXingResult.getOther();

            // if (pdf417RXingResultMetadata != null) {
            //   result.putMetadata(RXingResultMetadataType.PDF417_EXTRA_METADATA, pdf417RXingResultMetadata);
            // }

            result.putMetadata(
                RXingResultMetadataType::ORIENTATION,
                RXingResultMetadataValue::Orientation(detectorRXingResult.getRotation() as i32),
            );
            result.putMetadata(
                RXingResultMetadataType::SYMBOLOGY_IDENTIFIER,
                RXingResultMetadataValue::SymbologyIdentifier(format!(
                    "]L{}",
                    decoderRXingResult.getSymbologyModifier()
                )),
            );
            results.push(result);
        }
        Ok(results)
    }

    fn getMaxWidth(p1: &Option<RXingResultPoint>, p2: &Option<RXingResultPoint>) -> u64 {
        if let (Some(p1), Some(p2)) = (p1, p2) {
            (p1.getX() - p2.getX()).abs() as u64
        } else {
            0
        }
        // if p1 == null || p2 == null {
        //   return 0;
        // }
        // return (int) Math.abs(p1.getX() - p2.getX());
    }

    fn getMinWidth(p1: &Option<RXingResultPoint>, p2: &Option<RXingResultPoint>) -> u64 {
        if let (Some(p1), Some(p2)) = (p1, p2) {
            (p1.getX() - p2.getX()).abs() as u64
        } else {
            u32::MAX as u64
        }
        // if (p1 == null || p2 == null) {
        //   return Integer.MAX_VALUE;
        // }
        // return (int) Math.abs(p1.getX() - p2.getX());
    }

    fn getMaxCodewordWidth(p: &[Option<RXingResultPoint>]) -> u32 {
        Self::getMaxWidth(&p[0], &p[4])
            .max(
                Self::getMaxWidth(&p[6], &p[2]) * pdf_417_common::MODULES_IN_CODEWORD as u64
                    / pdf_417_common::MODULES_IN_STOP_PATTERN as u64,
            )
            .max(Self::getMaxWidth(&p[1], &p[5]).max(
                Self::getMaxWidth(&p[7], &p[3]) * pdf_417_common::MODULES_IN_CODEWORD as u64
                    / pdf_417_common::MODULES_IN_STOP_PATTERN as u64,
            )) as u32
    }

    fn getMinCodewordWidth(p: &[Option<RXingResultPoint>]) -> u32 {
        Self::getMinWidth(&p[0], &p[4])
            .min(
                Self::getMinWidth(&p[6], &p[2]) * pdf_417_common::MODULES_IN_CODEWORD as u64
                    / pdf_417_common::MODULES_IN_STOP_PATTERN as u64,
            )
            .min(Self::getMinWidth(&p[1], &p[5]).min(
                Self::getMinWidth(&p[7], &p[3]) * pdf_417_common::MODULES_IN_CODEWORD as u64
                    / pdf_417_common::MODULES_IN_STOP_PATTERN as u64,
            )) as u32
    }
}

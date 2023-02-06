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

use std::collections::HashMap;

use crate::{Exceptions, RXingResult, RXingResultPoint, Reader, ResultPoint};

/**
 * This class attempts to decode a barcode from an image, not by scanning the whole image,
 * but by scanning subsets of the image. This is important when there may be multiple barcodes in
 * an image, and detecting a barcode may find parts of multiple barcode and fail to decode
 * (e.g. QR Codes). Instead this scans the four quadrants of the image -- and also the center
 * 'quadrant' to cover the case where a barcode is found in the center.
 *
 * @see GenericMultipleBarcodeReader
 */
pub struct ByQuadrantReader<T: Reader>(T);
impl<T: Reader> Reader for ByQuadrantReader<T> {
    fn decode(
        &mut self,
        image: &mut crate::BinaryBitmap,
    ) -> Result<crate::RXingResult, crate::Exceptions> {
        self.decode_with_hints(image, &HashMap::new())
    }

    fn decode_with_hints(
        &mut self,
        image: &mut crate::BinaryBitmap,
        hints: &crate::DecodingHintDictionary,
    ) -> Result<crate::RXingResult, crate::Exceptions> {
        let width = image.getWidth();
        let height = image.getHeight();
        let halfWidth = width / 2;
        let halfHeight = height / 2;

        let attempt = self
            .0
            .decode_with_hints(&mut image.crop(0, 0, halfWidth, halfHeight), hints);
        // No need to call makeAbsolute as results will be relative to original top left here
        // This is a match because only NotFoundExceptions should be ignored
        match attempt {
            Err(Exceptions::NotFoundException(_)) => {}
            _ => return attempt,
        }

        // try {
        let result = self
            .0
            .decode_with_hints(&mut image.crop(halfWidth, 0, halfWidth, halfHeight), hints);
        // This is a match because only NotFoundExceptions should be ignored
        match result {
            Ok(res) => {
                let points = Self::makeAbsolute(res.getRXingResultPoints(), halfWidth as f32, 0.0);
                return Ok(RXingResult::new_from_existing_result(res, points));
            }
            Err(Exceptions::NotFoundException(_)) => {}
            _ => return result,
        }

        let result = self
            .0
            .decode_with_hints(&mut image.crop(0, halfHeight, halfWidth, halfHeight), hints);
        // This is a match because only NotFoundExceptions should be ignored
        match result {
            Ok(res) => {
                let points = Self::makeAbsolute(res.getRXingResultPoints(), 0.0, halfHeight as f32);
                return Ok(RXingResult::new_from_existing_result(res, points));
            }
            Err(Exceptions::NotFoundException(_)) => {}
            _ => return result,
        }

        let result = self.0.decode_with_hints(
            &mut image.crop(halfWidth, halfHeight, halfWidth, halfHeight),
            hints,
        );
        // This is a match because only NotFoundExceptions should be ignored
        match result {
            Ok(res) => {
                let points = Self::makeAbsolute(
                    res.getRXingResultPoints(),
                    halfWidth as f32,
                    halfHeight as f32,
                );
                return Ok(RXingResult::new_from_existing_result(res, points));
            }
            Err(Exceptions::NotFoundException(_)) => {}
            _ => return result,
        }

        let quarterWidth = halfWidth / 2;
        let quarterHeight = halfHeight / 2;
        let mut center = image.crop(quarterWidth, quarterHeight, halfWidth, halfHeight);
        let result = self.0.decode_with_hints(&mut center, hints)?;

        let points = Self::makeAbsolute(
            result.getRXingResultPoints(),
            quarterWidth as f32,
            quarterHeight as f32,
        );
        Ok(RXingResult::new_from_existing_result(result, points))
    }

    fn reset(&mut self) {
        self.0.reset()
    }
}

impl<T: Reader> ByQuadrantReader<T> {
    pub fn new(delegate: T) -> Self {
        Self(delegate)
    }

    fn makeAbsolute(
        points: &[RXingResultPoint],
        leftOffset: f32,
        topOffset: f32,
    ) -> Vec<RXingResultPoint> {
        // let mut result = Vec::new();
        // if !points.is_empty() {

        //     // for relative in points {
        //     //     result.push(RXingResultPoint::new(
        //     //         relative.getX() + leftOffset,
        //     //         relative.getY() + topOffset,
        //     //     ));
        //     // }
        // }
        // result
        points
            .iter()
            .map(|relative| {
                RXingResultPoint::new(relative.getX() + leftOffset, relative.getY() + topOffset)
            })
            .collect()
    }
}

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

// package com.google.zxing.common;

// import com.google.zxing.NotFoundException;

use crate::common::Result;
use crate::{point, Exceptions, Point};

use super::{BitMatrix, GridSampler, SamplerControl};

/**
 * @author Sean Owen
 */
#[derive(Default)]
pub struct DefaultGridSampler;

impl GridSampler for DefaultGridSampler {
    fn sample_grid(
        &self,
        image: &BitMatrix,
        dimensionX: u32,
        dimensionY: u32,
        controls: &[SamplerControl],
    ) -> Result<(BitMatrix, [Point; 4])> {
        if dimensionX == 0 || dimensionY == 0 {
            return Err(Exceptions::NOT_FOUND);
        }

        for SamplerControl { p0, p1, transform } in controls {
            // Precheck the corners of every roi to bail out early if the grid is "obviously" not completely inside the image
            let isInside = |x: f32, y: f32| {
                image.is_in(transform.transform_point(Point::centered(point(x, y))))
            };
            if !transform.isValid()
                || !isInside(p0.x, p0.y)
                || !isInside(p1.x - 1.0, p0.y)
                || !isInside(p1.x - 1.0, p1.y - 1.0)
                || !isInside(p0.x, p1.y - 1.0)
            {
                return Err(Exceptions::NOT_FOUND);
            }
        }

        let mut bits = BitMatrix::new(dimensionX, dimensionY)?;
        for SamplerControl { p0, p1, transform } in controls {
            // for (auto&& [x0, x1, y0, y1, mod2Pix] : rois) {
            for y in (p0.y as i32)..(p1.y as i32) {
                // for (int y = y0; y < y1; ++y)
                for x in (p0.x as i32)..(p1.x as i32) {
                    // for (int x = x0; x < x1; ++x) {
                    let p = transform.transform_point(Point::from((x, y)).centered()); //mod2Pix(centered(PointI{x, y}));

                    if image.get_point(p) {
                        bits.set(x as u32, y as u32);
                    }

                    // Due to a "numerical instability" in the PerspectiveTransform generation/application it has been observed
                    // that even though all boundary grid points get projected inside the image, it can still happen that an
                    // inner grid points is not. See #563. A true perspective transformation cannot have this property.
                    // The following check takes 100% care of the issue and turned out to be less of a performance impact than feared.
                    // TODO: Check some mathematical/numercial property of mod2Pix to determine if it is a perspective transforation.
                    if !image.is_in(p) {
                        return Err(Exceptions::NOT_FOUND);
                    }
                }
            }
        }

        // dbg!(image.to_string());
        // dbg!(bits.to_string());

        let projectCorner = |p: Point| -> Point {
            for SamplerControl { p0, p1, transform } in controls {
                if p0.x <= p.x && p.x <= p1.x && p0.y <= p.y && p.y <= p1.y {
                    return transform.transform_point(p) + point(0.5, 0.5);
                }
            }
            Point::default()
        };

        let tl = projectCorner(Point::default());
        let tr = projectCorner(Point::from((dimensionX, 0)));
        let bl = projectCorner(Point::from((dimensionX, dimensionY)));
        let br = projectCorner(Point::from((0, dimensionX)));

        Ok((bits, [tl, tr, bl, br]))
    }
}

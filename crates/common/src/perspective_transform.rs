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

/**
 * <p>This class implements a perspective transform in two dimensions. Given four source and four
 * destination points, it will compute the transformation implied between them. The code is based
 * directly upon section 3.4.2 of George Wolberg's "Digital Image Warping"; see pages 54-56.</p>
 *
 * @author Sean Owen
 */
pub struct PerspectiveTransform {
    a11: f32,
    a12: f32,
    a13: f32,
    a21: f32,
    a22: f32,
    a23: f32,
    a31: f32,
    a32: f32,
    a33: f32,
}

impl PerspectiveTransform {
    #[allow(clippy::too_many_arguments)]
    fn new(
        a11: f32,
        a21: f32,
        a31: f32,
        a12: f32,
        a22: f32,
        a32: f32,
        a13: f32,
        a23: f32,
        a33: f32,
    ) -> Self {
        Self {
            a11,
            a12,
            a13,
            a21,
            a22,
            a23,
            a31,
            a32,
            a33,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn quadrilateralToQuadrilateral(
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
        x0p: f32,
        y0p: f32,
        x1p: f32,
        y1p: f32,
        x2p: f32,
        y2p: f32,
        x3p: f32,
        y3p: f32,
    ) -> Self {
        let q_to_s = PerspectiveTransform::quadrilateralToSquare(x0, y0, x1, y1, x2, y2, x3, y3);
        let s_to_q =
            PerspectiveTransform::squareToQuadrilateral(x0p, y0p, x1p, y1p, x2p, y2p, x3p, y3p);
        s_to_q.times(&q_to_s)
    }

    pub fn transform_points_single(&self, points: &mut [f32]) {
        let a11 = self.a11;
        let a12 = self.a12;
        let a13 = self.a13;
        let a21 = self.a21;
        let a22 = self.a22;
        let a23 = self.a23;
        let a31 = self.a31;
        let a32 = self.a32;
        let a33 = self.a33;
        let maxI = points.len() - 1; // points.length must be even
        let mut i = 0;
        while i < maxI {
            // for (int i = 0; i < maxI; i += 2) {
            let x = points[i];
            let y = points[i + 1];
            let denominator = a13 * x + a23 * y + a33;
            points[i] = (a11 * x + a21 * y + a31) / denominator;
            points[i + 1] = (a12 * x + a22 * y + a32) / denominator;
            i += 2;
        }
    }

    pub fn transform_points_double(&self, x_values: &mut [f32], y_valuess: &mut [f32]) {
        let n = x_values.len();
        // for i in 0..n {
        for (x, y) in x_values.iter_mut().zip(y_valuess.iter_mut()).take(n) {
            // for (int i = 0; i < n; i++) {
            let denominator = self.a13 * *x + self.a23 * *y + self.a33;
            *x = (self.a11 * *x + self.a21 * *y + self.a31) / denominator;
            *y = (self.a12 * *x + self.a22 * *y + self.a32) / denominator;
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn squareToQuadrilateral(
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
    ) -> Self {
        let dx3 = x0 - x1 + x2 - x3;
        let dy3 = y0 - y1 + y2 - y3;
        if dx3 == 0.0 && dy3 == 0.0 {
            // Affine
            PerspectiveTransform::new(x1 - x0, x2 - x1, x0, y1 - y0, y2 - y1, y0, 0.0, 0.0, 1.0)
        } else {
            let dx1 = x1 - x2;
            let dx2 = x3 - x2;
            let dy1 = y1 - y2;
            let dy2 = y3 - y2;
            let denominator = dx1 * dy2 - dx2 * dy1;
            let a13 = (dx3 * dy2 - dx2 * dy3) / denominator;
            let a23 = (dx1 * dy3 - dx3 * dy1) / denominator;
            PerspectiveTransform::new(
                x1 - x0 + a13 * x1,
                x3 - x0 + a23 * x3,
                x0,
                y1 - y0 + a13 * y1,
                y3 - y0 + a23 * y3,
                y0,
                a13,
                a23,
                1.0,
            )
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn quadrilateralToSquare(
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
    ) -> Self {
        // Here, the adjoint serves as the inverse
        PerspectiveTransform::squareToQuadrilateral(x0, y0, x1, y1, x2, y2, x3, y3).buildAdjoint()
    }

    fn buildAdjoint(&self) -> Self {
        // Adjoint is the transpose of the cofactor matrix:
        PerspectiveTransform::new(
            self.a22 * self.a33 - self.a23 * self.a32,
            self.a23 * self.a31 - self.a21 * self.a33,
            self.a21 * self.a32 - self.a22 * self.a31,
            self.a13 * self.a32 - self.a12 * self.a33,
            self.a11 * self.a33 - self.a13 * self.a31,
            self.a12 * self.a31 - self.a11 * self.a32,
            self.a12 * self.a23 - self.a13 * self.a22,
            self.a13 * self.a21 - self.a11 * self.a23,
            self.a11 * self.a22 - self.a12 * self.a21,
        )
    }

    fn times(&self, other: &Self) -> Self {
        PerspectiveTransform::new(
            self.a11 * other.a11 + self.a21 * other.a12 + self.a31 * other.a13,
            self.a11 * other.a21 + self.a21 * other.a22 + self.a31 * other.a23,
            self.a11 * other.a31 + self.a21 * other.a32 + self.a31 * other.a33,
            self.a12 * other.a11 + self.a22 * other.a12 + self.a32 * other.a13,
            self.a12 * other.a21 + self.a22 * other.a22 + self.a32 * other.a23,
            self.a12 * other.a31 + self.a22 * other.a32 + self.a32 * other.a33,
            self.a13 * other.a11 + self.a23 * other.a12 + self.a33 * other.a13,
            self.a13 * other.a21 + self.a23 * other.a22 + self.a33 * other.a23,
            self.a13 * other.a31 + self.a23 * other.a32 + self.a33 * other.a33,
        )
    }
}

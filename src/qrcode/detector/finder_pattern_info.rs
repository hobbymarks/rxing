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

use super::FinderPattern;

/**
 * <p>Encapsulates information about finder patterns in an image, including the location of
 * the three finder patterns, and their estimated module size.</p>
 *
 * @author Sean Owen
 */
pub struct FinderPatternInfo {
    bottomLeft: FinderPattern,
    topLeft: FinderPattern,
    topRight: FinderPattern,
}

impl FinderPatternInfo {
    /// Expects the order to be [bottom_left, top_left, top_right]
    pub fn new(patternCenters: [FinderPattern; 3]) -> Self {
        let [a, b, c] = patternCenters;
        Self {
            bottomLeft: a,
            topLeft: b,
            topRight: c,
        }
    }

    pub fn getBottomLeft(&self) -> &FinderPattern {
        &self.bottomLeft
    }

    pub fn getTopLeft(&self) -> &FinderPattern {
        &self.topLeft
    }

    pub fn getTopRight(&self) -> &FinderPattern {
        &self.topRight
    }
}

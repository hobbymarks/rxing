/*
 * Copyright 2013 ZXing authors
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

/**
 * @author Guenther Grau
 */
#[derive(Clone, Default)]
pub struct BarcodeValue(HashMap<u32, u32>);

impl BarcodeValue {
    pub fn new() -> Self {
        Self::default()
    }

    /**
     * Add an occurrence of a value
     */
    pub fn setValue(&mut self, value: u32) {
        self.0
            .entry(value)
            .and_modify(|confidence| *confidence += 1)
            .or_insert(1);
    }

    /**
     * Determines the maximum occurrence of a set value and returns all values which were set with this occurrence.
     * @return an array of int, containing the values with the highest occurrence, or null, if no value was set
     */
    pub fn getValue(&self) -> Vec<u32> {
        let mut maxConfidence = -1_i32;
        let mut result = Vec::new();
        for (key, value) in &self.0 {
            match (*value as i32).cmp(&maxConfidence) {
                std::cmp::Ordering::Greater => {
                    maxConfidence = *value as i32;
                    result.clear();
                    result.push(*key);
                }
                std::cmp::Ordering::Equal => result.push(*key),
                std::cmp::Ordering::Less => {}
            }
        }

        result
    }

    pub fn getConfidence(&self, value: u32) -> u32 {
        *self.0.get(&value).unwrap_or(&0)
    }
}

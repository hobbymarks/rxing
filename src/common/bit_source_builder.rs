/*
 * Copyright 2008 ZXing authors
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

// import java.io.ByteArrayOutputStream;

use std::io::Write;

/**
 * Class that lets one easily build an array of bytes by appending bits at a time.
 *
 * @author Sean Owen
 */
pub struct BitSourceBuilder {
    output: Vec<u8>,
    nextByte: u32,
    bitsLeftInNextByte: u32,
}

impl BitSourceBuilder {
    pub const fn new() -> Self {
        Self {
            output: Vec::new(),
            nextByte: 0,
            bitsLeftInNextByte: 8,
        }
    }

    pub fn write(&mut self, value: u32, numBits: u32) {
        if numBits <= self.bitsLeftInNextByte {
            self.nextByte <<= numBits;
            self.nextByte |= value;
            self.bitsLeftInNextByte -= numBits;
            if self.bitsLeftInNextByte == 0 {
                self.output.push(self.nextByte as u8);
                self.nextByte = 0;
                self.bitsLeftInNextByte = 8;
            }
        } else {
            let bitsToWriteNow = self.bitsLeftInNextByte;
            let numRestOfBits = numBits - bitsToWriteNow;
            let mask = 0xFF >> (8 - bitsToWriteNow);
            let valueToWriteNow = (value >> numRestOfBits) & mask;
            self.write(valueToWriteNow, bitsToWriteNow);
            self.write(value, numRestOfBits);
        }
    }

    pub fn asByteArray(&mut self) -> &Vec<u8> {
        if self.bitsLeftInNextByte < 8 {
            self.write(0, self.bitsLeftInNextByte);
        }
        &self.output
    }

    pub fn toByteArray(mut self) -> Vec<u8> {
        if self.bitsLeftInNextByte < 8 {
            self.write(0, self.bitsLeftInNextByte);
        }
        self.output
    }
}

impl Default for BitSourceBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Write for BitSourceBuilder {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut written = 0;
        for byte in buf.iter() {
            self.write(*byte as u32, 8);
            written += 1;
        }

        Ok(written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

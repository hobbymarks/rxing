/*
 * Copyright 2006-2007 Jeremias Maerki.
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

use crate::common::Result;
use crate::Exceptions;

use super::{
    high_level_encoder::{self, ASCII_ENCODATION, BASE256_ENCODATION},
    Encoder,
};

#[derive(Debug, Default)]
pub struct Base256Encoder;

impl Encoder for Base256Encoder {
    fn getEncodingMode(&self) -> usize {
        BASE256_ENCODATION
    }

    fn encode(&self, context: &mut super::EncoderContext) -> Result<()> {
        let mut buffer = String::new();
        buffer.push('\0'); //Initialize length field
        while context.hasMoreCharacters() {
            let c = context.getCurrentChar();
            buffer.push(c);

            context.pos += 1;

            let newMode = high_level_encoder::lookAheadTest(
                context.getMessage(),
                context.pos,
                self.getEncodingMode() as u32,
            );
            if newMode != self.getEncodingMode() {
                // Return to ASCII encodation, which will actually handle latch to new mode
                context.signalEncoderChange(ASCII_ENCODATION);
                break;
            }
        }
        let dataCount = buffer.chars().count() - 1;
        let lengthFieldSize = 1;
        let currentSize = context.getCodewordCount() + dataCount + lengthFieldSize;
        context.updateSymbolInfoWithLength(currentSize);
        let mustPad = (context
            .getSymbolInfo()
            .ok_or(Exceptions::ILLEGAL_STATE)?
            .getDataCapacity()
            - currentSize as u32)
            > 0;
        if context.hasMoreCharacters() || mustPad {
            if dataCount <= 249 {
                buffer.replace_range(
                    0..1,
                    &char::from_u32(dataCount as u32)
                        .ok_or(Exceptions::PARSE)?
                        .to_string(),
                );
            } else if dataCount <= 1555 {
                buffer.replace_range(
                    0..1,
                    &char::from_u32((dataCount as u32 / 250) + 249)
                        .ok_or(Exceptions::PARSE)?
                        .to_string(),
                );
                let (ci_pos, _) = buffer
                    .char_indices()
                    .nth(1)
                    .ok_or(Exceptions::INDEX_OUT_OF_BOUNDS)?;
                buffer.insert(
                    ci_pos,
                    char::from_u32(dataCount as u32 % 250)
                        .ok_or(Exceptions::INDEX_OUT_OF_BOUNDS)?,
                );
            } else {
                return Err(Exceptions::illegal_state_with(format!(
                    "Message length not in valid ranges: {dataCount}"
                )));
            }
        }
        for buffer_char in buffer.chars() {
            context.writeCodeword(
                Self::randomize255State(buffer_char, context.getCodewordCount() as u32 + 1)
                    .ok_or(Exceptions::PARSE)? as u8,
            );
        }
        Ok(())
    }
}
impl Base256Encoder {
    pub fn new() -> Self {
        Self
    }
    const fn randomize255State(ch: char, codewordPosition: u32) -> Option<char> {
        let pseudoRandom = ((149 * codewordPosition) % 255) + 1;
        let tempVariable = ch as u32 + pseudoRandom;
        if tempVariable <= 255 {
            char::from_u32(tempVariable)
        } else {
            char::from_u32(tempVariable - 256)
        }
    }
}

/*
 * Copyright 2021 ZXing authors
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

use unicode_segmentation::UnicodeSegmentation;

use super::{CharacterSet, Eci};

// static ENCODERS: Lazy<Vec<CharacterSet>> = Lazy::new(|| {
//     let mut enc_vec = Vec::new();
//     for name in NAMES {
//         if let Some(enc) = CharacterSet::get_character_set_by_name(name) {
//             enc_vec.push(enc);
//         }
//     }
//     enc_vec
// });

// const NAMES: [&str; 20] = [
//     "IBM437",
//     "ISO-8859-2",
//     "ISO-8859-3",
//     "ISO-8859-4",
//     "ISO-8859-5",
//     "ISO-8859-6",
//     "ISO-8859-7",
//     "ISO-8859-8",
//     "ISO-8859-9",
//     "ISO-8859-10",
//     "ISO-8859-11",
//     "ISO-8859-13",
//     "ISO-8859-14",
//     "ISO-8859-15",
//     "ISO-8859-16",
//     "windows-1250",
//     "windows-1251",
//     "windows-1252",
//     "windows-1256",
//     "Shift_JIS",
// ];

const ENCODERS: [CharacterSet; 14] = [
    CharacterSet::Cp437,
    CharacterSet::ISO8859_2,
    CharacterSet::ISO8859_3,
    CharacterSet::ISO8859_4,
    CharacterSet::ISO8859_5,
    // CharacterSet::ISO8859_6,
    CharacterSet::ISO8859_7,
    // CharacterSet::ISO8859_8,
    CharacterSet::ISO8859_9,
    // CharacterSet::ISO8859_10,
    // CharacterSet::ISO8859_11,
    // CharacterSet::ISO8859_13,
    // CharacterSet::ISO8859_14,
    CharacterSet::ISO8859_15,
    CharacterSet::ISO8859_16,
    CharacterSet::Shift_JIS,
    CharacterSet::Cp1250,
    CharacterSet::Cp1251,
    CharacterSet::Cp1252,
    CharacterSet::Cp1256,
];

/**
 * Set of CharsetEncoders for a given input string
 *
 * Invariants:
 * - The list contains only encoders from CharacterSetECI (list is shorter then the list of encoders available on
 *   the platform for which ECI values are defined).
 * - The list contains encoders at least one encoder for every character in the input.
 * - The first encoder in the list is always the ISO-8859-1 encoder even of no character in the input can be encoded
 *   by it.
 * - If the input contains a character that is not in ISO-8859-1 then the last two entries in the list will be the
 *   UTF-8 encoder and the UTF-16BE encoder.
 *
 * @author Alex Geller
 */
#[derive(Clone)]
pub struct ECIEncoderSet {
    encoders: Vec<CharacterSet>,
    priorityEncoderIndex: Option<usize>,
}

impl ECIEncoderSet {
    /**
     * Constructs an encoder set
     *
     * @param stringToEncode the string that needs to be encoded
     * @param priorityCharset The preferred {@link Charset} or null.
     * @param fnc1 fnc1 denotes the character in the input that represents the FNC1 character or -1 for a non-GS1 bar
     * code. When specified, it is considered an error to pass it as argument to the methods canEncode() or encode().
     */
    pub fn new(
        stringToEncodeMain: &str,
        priorityCharset: Option<CharacterSet>,
        fnc1: Option<&str>,
    ) -> Self {
        // List of encoders that potentially encode characters not in ISO-8859-1 in one byte.

        let mut encoders: Vec<CharacterSet>;
        let mut priorityEncoderIndexValue = None;

        let mut neededEncoders: Vec<CharacterSet> = Vec::new();

        let stringToEncode = stringToEncodeMain.graphemes(true).collect::<Vec<&str>>();

        //we always need the ISO-8859-1 encoder. It is the default encoding
        neededEncoders.push(CharacterSet::ISO8859_1);
        let mut needUnicodeEncoder = if let Some(pc) = priorityCharset {
            //pc.name().starts_with("UTF") || pc.name().starts_with("utf")
            pc == CharacterSet::UTF8 || pc == CharacterSet::UTF16BE
        } else {
            false
        };

        //Walk over the input string and see if all characters can be encoded with the list of encoders
        for i in 0..stringToEncode.len() {
            // for (int i = 0; i < stringToEncode.length(); i++) {
            let mut canEncode = false;
            for encoder in &neededEncoders {
                //   for (CharsetEncoder encoder : neededEncoders) {
                let c = stringToEncode.get(i).unwrap();
                if (fnc1.is_some() && c == fnc1.as_ref().unwrap()) || encoder.encode(c).is_ok() {
                    canEncode = true;
                    break;
                }
            }
            if !canEncode {
                //for the character at position i we don't yet have an encoder in the list
                for encoder in ENCODERS.iter() {
                    if encoder.encode(stringToEncode.get(i).unwrap()).is_ok() {
                        //Good, we found an encoder that can encode the character. We add him to the list and continue scanning
                        //the input
                        neededEncoders.push(*encoder);
                        canEncode = true;
                        break;
                    }
                }
            }

            if !canEncode {
                //The character is not encodeable by any of the single byte encoders so we remember that we will need a
                //Unicode encoder.
                needUnicodeEncoder = true;
            }
        }

        if neededEncoders.len() == 1 && !needUnicodeEncoder {
            //the entire input can be encoded by the ISO-8859-1 encoder
            encoders = vec![CharacterSet::ISO8859_1];
        } else {
            // we need more than one single byte encoder or we need a Unicode encoder.
            // In this case we append a UTF-8 and UTF-16 encoder to the list
            //   encoders = [] new CharsetEncoder[neededEncoders.size() + 2];
            encoders = Vec::with_capacity(neededEncoders.len() + 2);

            encoders.extend(neededEncoders);

            encoders.push(CharacterSet::UTF8);
            encoders.push(CharacterSet::UTF16BE);
        }

        //Compute priorityEncoderIndex by looking up priorityCharset in encoders
        if let Some(pc) = priorityCharset.as_ref() {
            priorityEncoderIndexValue = encoders.iter().position(|enc| enc == pc);
        }
        //invariants
        assert_eq!(encoders[0], CharacterSet::ISO8859_1);
        Self {
            encoders,
            priorityEncoderIndex: priorityEncoderIndexValue,
        }
    }

    pub const fn len(&self) -> usize {
        self.encoders.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.encoders.is_empty()
    }

    pub fn getCharsetName(&self, index: usize) -> Option<&'static str> {
        if index < self.len() {
            Some(self.encoders[index].get_charset_name())
        } else {
            None
        }
    }

    pub fn getCharset(&self, index: usize) -> Option<CharacterSet> {
        if index < self.len() {
            Some(self.encoders[index])
        } else {
            None
        }
    }

    pub fn get_eci(&self, encoderIndex: usize) -> Eci {
        self.encoders[encoderIndex].into()
        // CharacterSetECI::getValue(
        //     &CharacterSetECI::getCharacterSetECI(self.encoders[encoderIndex]).unwrap(),
        // )
    }

    /*
     *  returns -1 if no priority charset was defined
     */
    pub const fn getPriorityEncoderIndex(&self) -> Option<usize> {
        self.priorityEncoderIndex
    }

    pub fn canEncode(&self, c: &str, encoderIndex: usize) -> Option<bool> {
        if encoderIndex < self.len() {
            let encoder = self.encoders[encoderIndex];
            let enc_data = encoder.encode(c);

            Some(enc_data.is_ok())
        } else {
            None
        }
    }

    pub fn encode_char(&self, c: &str, encoderIndex: usize) -> Option<Vec<u8>> {
        if encoderIndex < self.len() {
            let encoder = self.encoders[encoderIndex];
            let enc_data = encoder.encode(c);
            enc_data.ok()
        // assert!(enc_data.is_ok());
        // enc_data.unwrap()
        } else {
            None
        }
    }

    pub fn encode_string(&self, s: &str, encoderIndex: usize) -> Option<Vec<u8>> {
        if encoderIndex < self.len() {
            let encoder = self.encoders[encoderIndex];
            encoder.encode(s).ok()
        } else {
            None
        }
    }
}

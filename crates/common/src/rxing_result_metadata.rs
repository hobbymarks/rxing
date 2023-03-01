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

//package com.google.zxing;

use std::rc::Rc;

use crate::PDF417RXingResultMetadata;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/**
 * Represents some type of metadata about the result of the decoding that the decoder
 * wishes to communicate back to the caller.
 *
 * @author Sean Owen
 */
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Hash, Debug, Clone)]
pub enum RXingResultMetadataType {
    /**
     * Unspecified, application-specific metadata. Maps to an unspecified {@link Object}.
     */
    OTHER,

    /**
     * Denotes the likely approximate orientation of the barcode in the image. This value
     * is given as degrees rotated clockwise from the normal, upright orientation.
     * For example a 1D barcode which was found by reading top-to-bottom would be
     * said to have orientation "90". This key maps to an {@link Integer} whose
     * value is in the range [0,360).
     */
    ORIENTATION,

    /**
     * <p>2D barcode formats typically encode text, but allow for a sort of 'byte mode'
     * which is sometimes used to encode binary data. While {@link RXingResult} makes available
     * the complete raw bytes in the barcode for these formats, it does not offer the bytes
     * from the byte segments alone.</p>
     *
     * <p>This maps to a {@link java.util.List} of byte arrays corresponding to the
     * raw bytes in the byte segments in the barcode, in order.</p>
     */
    BYTE_SEGMENTS,

    /**
     * Error correction level used, if applicable. The value type depends on the
     * format, but is typically a String.
     */
    ERROR_CORRECTION_LEVEL,

    /**
     * For some periodicals, indicates the issue number as an {@link Integer}.
     */
    ISSUE_NUMBER,

    /**
     * For some products, indicates the suggested retail price in the barcode as a
     * formatted {@link String}.
     */
    SUGGESTED_PRICE,

    /**
     * For some products, the possible country of manufacture as a {@link String} denoting the
     * ISO country code. Some map to multiple possible countries, like "US/CA".
     */
    POSSIBLE_COUNTRY,

    /**
     * For some products, the extension text
     */
    UPC_EAN_EXTENSION,

    /**
     * PDF417-specific metadata
     */
    PDF417_EXTRA_METADATA,

    /**
     * If the code format supports structured append and the current scanned code is part of one then the
     * sequence number is given with it.
     */
    STRUCTURED_APPEND_SEQUENCE,

    /**
     * If the code format supports structured append and the current scanned code is part of one then the
     * parity is given with it.
     */
    STRUCTURED_APPEND_PARITY,

    /**
     * Barcode Symbology Identifier.
     * Note: According to the GS1 specification the identifier may have to replace a leading FNC1/GS character
     * when prepending to the barcode content.
     */
    SYMBOLOGY_IDENTIFIER,

    IS_MIRRORED,

    CONTENT_TYPE,
}

impl From<String> for RXingResultMetadataType {
    fn from(in_str: String) -> Self {
        match in_str.to_uppercase().as_str() {
            "OTHER" => RXingResultMetadataType::OTHER,
            "ORIENTATION" => RXingResultMetadataType::ORIENTATION,
            "BYTE_SEGMENTS" | "BYTESEGMENTS" => RXingResultMetadataType::BYTE_SEGMENTS,
            "ERROR_CORRECTION_LEVEL" | "ERRORCORRECTIONLEVEL" => {
                RXingResultMetadataType::ERROR_CORRECTION_LEVEL
            }
            "ISSUE_NUMBER" | "ISSUENUMBER" => RXingResultMetadataType::ISSUE_NUMBER,
            "SUGGESTED_PRICE" | "SUGGESTEDPRICE" => RXingResultMetadataType::SUGGESTED_PRICE,
            "POSSIBLE_COUNTRY" | "POSSIBLECOUNTRY" => RXingResultMetadataType::POSSIBLE_COUNTRY,
            "UPC_EAN_EXTENSION" | "UPCEANEXTENSION" => RXingResultMetadataType::UPC_EAN_EXTENSION,
            "PDF417_EXTRA_METADATA" | "PDF417EXTRAMETADATA" => {
                RXingResultMetadataType::PDF417_EXTRA_METADATA
            }
            "STRUCTURED_APPEND_SEQUENCE" | "STRUCTUREDAPPENDSEQUENCE" => {
                RXingResultMetadataType::STRUCTURED_APPEND_SEQUENCE
            }
            "STRUCTURED_APPEND_PARITY" | "STRUCTUREDAPPENDPARITY" => {
                RXingResultMetadataType::STRUCTURED_APPEND_PARITY
            }
            "SYMBOLOGY_IDENTIFIER" | "SYMBOLOGYIDENTIFIER" => {
                RXingResultMetadataType::SYMBOLOGY_IDENTIFIER
            }
            "IS_MIRRORED" | "ISMIRRORED" => RXingResultMetadataType::IS_MIRRORED,
            "CONTENT_TYPE" | "CONTENTTYPE" => RXingResultMetadataType::CONTENT_TYPE,
            _ => RXingResultMetadataType::OTHER,
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RXingResultMetadataValue {
    /**
     * Unspecified, application-specific metadata. Maps to an unspecified {@link Object}.
     */
    OTHER(String),

    /**
     * Denotes the likely approximate orientation of the barcode in the image. This value
     * is given as degrees rotated clockwise from the normal, upright orientation.
     * For example a 1D barcode which was found by reading top-to-bottom would be
     * said to have orientation "90". This key maps to an {@link Integer} whose
     * value is in the range [0,360).
     */
    Orientation(i32),

    /**
     * <p>2D barcode formats typically encode text, but allow for a sort of 'byte mode'
     * which is sometimes used to encode binary data. While {@link RXingResult} makes available
     * the complete raw bytes in the barcode for these formats, it does not offer the bytes
     * from the byte segments alone.</p>
     *
     * <p>This maps to a {@link java.util.List} of byte arrays corresponding to the
     * raw bytes in the byte segments in the barcode, in order.</p>
     */
    ByteSegments(Vec<Vec<u8>>),

    /**
     * Error correction level used, if applicable. The value type depends on the
     * format, but is typically a String.
     */
    ErrorCorrectionLevel(String),

    /**
     * For some periodicals, indicates the issue number as an {@link Integer}.
     */
    IssueNumber(i32),

    /**
     * For some products, indicates the suggested retail price in the barcode as a
     * formatted {@link String}.
     */
    SuggestedPrice(String),

    /**
     * For some products, the possible country of manufacture as a {@link String} denoting the
     * ISO country code. Some map to multiple possible countries, like "US/CA".
     */
    PossibleCountry(String),

    /**
     * For some products, the extension text
     */
    UpcEanExtension(String),

    /**
     * PDF417-specific metadata
     */
    Pdf417ExtraMetadata(Rc<PDF417RXingResultMetadata>),

    /**
     * If the code format supports structured append and the current scanned code is part of one then the
     * sequence number is given with it.
     */
    StructuredAppendSequence(i32),

    /**
     * If the code format supports structured append and the current scanned code is part of one then the
     * parity is given with it.
     */
    StructuredAppendParity(i32),

    /**
     * Barcode Symbology Identifier.
     * Note: According to the GS1 specification the identifier may have to replace a leading FNC1/GS character
     * when prepending to the barcode content.
     */
    SymbologyIdentifier(String),

    IsMirrored(bool),

    ContentType(String),
}

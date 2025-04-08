#![allow(dead_code)]
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

use std::{
    collections::HashMap,
    fs::{read_dir, read_to_string, File},
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
};

use rxing::{
    common::{CharacterSet, HybridBinarizer, Result},
    multi::MultipleBarcodeReader,
    pdf417::PDF417RXingResultMetadata,
    BarcodeFormat, Binarizer, BinaryBitmap, BufferedImageLuminanceSource, DecodeHintType,
    DecodeHintValue, DecodeHints, RXingResult, RXingResultMetadataType, RXingResultMetadataValue,
    Reader,
};

use super::TestRXingResult;

/**
 * @author Sean Owen
 * @author dswitkin@google.com (Daniel Switkin)
 */
pub struct MultiImageSpanAbstractBlackBoxTestCase<T: MultipleBarcodeReader + Reader> {
    test_base: Box<Path>,
    barcode_reader: T,
    expected_format: BarcodeFormat,
    test_rxing_results: Vec<TestRXingResult>,
    hints: DecodeHints,
}

impl<T: MultipleBarcodeReader + Reader> MultiImageSpanAbstractBlackBoxTestCase<T> {
    pub fn build_test_base(test_base_path_suffix: &str) -> Box<Path> {
        // A little workaround to prevent aggravation in my IDE
        let test_base = Path::new(test_base_path_suffix);
        // if !testBase.exists() {
        //   // try starting with 'core' since the test base is often given as the project root
        //   testBase = Paths.get("core").resolve(testBasePathSuffix);
        // }
        test_base.to_owned().into()
    }

    pub fn new(
        test_base_path_suffix: &str,
        barcode_reader: T,
        expected_format: BarcodeFormat,
    ) -> Self {
        Self {
            test_base: Self::build_test_base(test_base_path_suffix),
            barcode_reader,
            expected_format,
            test_rxing_results: Vec::new(),
            hints: DecodeHints::default(),
        }
    }

    pub fn get_test_base(&self) -> &Box<Path> {
        &self.test_base
    }

    pub fn add_test(&mut self, must_pass_count: u32, try_harder_count: u32, rotation: f32) {
        self.add_test_complex(must_pass_count, try_harder_count, 0, 0, rotation);
    }

    pub fn add_hint(&mut self, _hint: DecodeHintType, value: DecodeHintValue) {
        match value {
            DecodeHintValue::Other(v) => self.hints.Other = Some(v),
            DecodeHintValue::PureBarcode(v) => self.hints.PureBarcode = Some(v),
            DecodeHintValue::PossibleFormats(v) => self.hints.PossibleFormats = Some(v),
            DecodeHintValue::TryHarder(v) => self.hints.TryHarder = Some(v),
            DecodeHintValue::CharacterSet(v) => self.hints.CharacterSet = Some(v),
            DecodeHintValue::AllowedLengths(v) => self.hints.AllowedLengths = Some(v),
            DecodeHintValue::AssumeCode39CheckDigit(v) => {
                self.hints.AssumeCode39CheckDigit = Some(v)
            }
            DecodeHintValue::AssumeGs1(v) => self.hints.AssumeGs1 = Some(v),
            DecodeHintValue::ReturnCodabarStartEnd(v) => self.hints.ReturnCodabarStartEnd = Some(v),
            DecodeHintValue::NeedResultPointCallback(v) => {
                self.hints.NeedResultPointCallback = Some(v)
            }
            DecodeHintValue::AllowedEanExtensions(v) => self.hints.AllowedEanExtensions = Some(v),
            DecodeHintValue::AlsoInverted(v) => self.hints.AlsoInverted = Some(v),
            DecodeHintValue::TelepenAsNumeric(v) => self.hints.TelepenAsNumeric = Some(v),
            #[cfg(feature = "allow_forced_iso_ied_18004_compliance")]
            DecodeHintValue::QrAssumeSpecConformInput(v) => {
                self.hints.QrAssumeSpecConformInput = Some(v)
            }
        }
    }

    /**
     * Adds a new test for the current directory of images.
     *
     * @param mustPassCount The number of images which must decode for the test to pass.
     * @param tryHarderCount The number of images which must pass using the try harder flag.
     * @param maxMisreads Maximum number of images which can fail due to successfully reading the wrong contents
     * @param maxTryHarderMisreads Maximum number of images which can fail due to successfully
     *                             reading the wrong contents using the try harder flag
     * @param rotation The rotation in degrees clockwise to use for this test.
     */
    pub fn add_test_complex(
        &mut self,
        must_pass_count: u32,
        try_harder_count: u32,
        max_misreads: u32,
        max_try_harder_misreads: u32,
        rotation: f32,
    ) {
        self.test_rxing_results.push(TestRXingResult::new(
            must_pass_count,
            try_harder_count,
            max_misreads,
            max_try_harder_misreads,
            rotation,
        ));
    }

    pub fn get_image_files(&self) -> Vec<PathBuf> {
        assert!(
            self.test_base.exists(),
            "Please download and install test images, and run from the 'core' directory"
        );
        // let paths = Vec::new();
        let path_search = read_dir(&self.test_base);
        const POSSIBLE_EXTENSIONS: &str = "jpg,jpeg,gif,png,JPG,JPEG,GIF,PNG";

        let mut paths = path_search
            .unwrap()
            .filter(|r| r.is_ok()) // Get rid of Err variants for Result<DirEntry>
            .map(|r| r.unwrap().path()) // This is safe, since we only have the Ok variants
            .filter(|r| r.is_file()) // Filter out non-folders
            .filter(|r| POSSIBLE_EXTENSIONS.contains(r.extension().unwrap().to_str().unwrap()))
            // .map(|r| r.into_boxed_path())
            .collect::<Vec<PathBuf>>();

        paths.sort();

        paths
    }

    pub fn get_reader(&self) -> &T {
        &self.barcode_reader
    }

    pub fn test_black_box(&mut self) {
        assert!(!self.test_rxing_results.is_empty());

        let image_files = self.get_image_file_lists().expect("images");
        let test_count = self.test_rxing_results.len();

        let mut passed_counts = vec![0; test_count];
        let mut try_harder_counts = vec![0; test_count];

        let test_base = self.get_test_base().clone();

        for (name, files) in &image_files {
            // for (Entry<String,List<Path>> testImageGroup : imageFiles.entrySet()) {
            log::fine(format!("Starting Image Group {name}"));

            let file_base_name = name; //testImageGroup.getKey();
                                       //   let expectedText : String;
            let mut expected_text_file = test_base.clone().to_path_buf();
            expected_text_file.push(file_base_name);
            expected_text_file.set_extension("txt");
            //   let expectedTextFile = testBase.resolve(fileBaseName + ".txt");
            let expected_text = if expected_text_file.exists() {
                Self::read_file_as_string(expected_text_file)
            } else {
                let mut new_path = self.test_base.clone().to_path_buf();
                new_path.push(file_base_name);
                new_path.set_extension("bin");
                //expectedTextFile = testBase.resolve(fileBaseName + ".bin");
                assert!(new_path.exists());
                Self::read_file_as_string(new_path)
            }
            .unwrap();

            for x in 0..test_count {
                //   for (int x = 0; x < testCount; x++) {
                let mut results = Vec::new();
                for image_file in files {
                    // for (Path imageFile : testImageGroup.getValue()) {
                    let image = image::open(image_file).unwrap();
                    let rotation: f32 = self.test_rxing_results.get(x).expect("ok").get_rotation();
                    let rotated_image = Self::rotate_image(&image, rotation);
                    let source = BufferedImageLuminanceSource::new(rotated_image);
                    let mut bitmap = BinaryBitmap::new(HybridBinarizer::new(source));

                    if let Ok(res) =
                        Self::decode_pdf417(&mut bitmap, false, &mut self.barcode_reader)
                    {
                        for r in res {
                            results.push(r);
                        }
                    }
                    //   try {
                    //     results.addAll(Arrays.asList(decode(bitmap, false)));
                    //   } catch (ReaderException ignored) {
                    //     // ignore
                    //   }
                }
                // results.sort(Comparator.comparingInt((RXingResult r) -> getMeta(r).getSegmentIndex()));
                // results.sort();
                let mut result_text = String::new(); //new StringBuilder();
                let mut file_id: Option<String> = None;
                if self.expected_format == BarcodeFormat::PDF_417 {
                    for result in results {
                        // for (RXingResult result : results) {
                        let result_metadata = Self::get_meta(&result);
                        assert!(result_metadata.is_some(), "resultMetadata");
                        if file_id.is_none() {
                            file_id =
                                Some(result_metadata.as_ref().unwrap().getFileId().to_owned());
                        }
                        assert_eq!(
                            file_id,
                            Some(result_metadata.as_ref().unwrap().getFileId().to_owned()),
                            "FileId"
                        );
                        result_text.push_str(result.getText());
                    }
                } else if self.expected_format != BarcodeFormat::PDF_417 {
                    results.sort_by_key(|r| {
                        if let Some(RXingResultMetadataValue::StructuredAppendSequence(md)) = r
                            .getRXingResultMetadata()
                            .get(&RXingResultMetadataType::STRUCTURED_APPEND_SEQUENCE)
                        {
                            *md
                        } else {
                            0
                        }
                    });
                    for result in results {
                        result_text.push_str(result.getText());
                    }
                }
                assert_eq!(expected_text, result_text, "ExpectedText");
                passed_counts[x] += 1;
                try_harder_counts[x] += 1;
            }
        }

        // Print the results of all tests first
        let mut total_found = 0;
        let mut total_must_pass = 0;

        let number_of_tests = image_files.len(); //imageFiles.keySet().size();
        for x in 0..self.test_rxing_results.len() {
            // for (int x = 0; x < testRXingResults.size(); x++) {
            let test_rxing_result = self.test_rxing_results.get(x).expect("ok");
            log::info(format!(
                "Rotation {} degrees:",
                test_rxing_result.get_rotation()
            ));
            log::info(format!(
                " {} of {} images passed ({} required)",
                passed_counts[x],
                number_of_tests,
                test_rxing_result.get_must_pass_count()
            ));
            log::info(format!(
                " {} of {} images passed with try harder ({} required)",
                try_harder_counts[x],
                number_of_tests,
                test_rxing_result.get_try_harder_count()
            ));
            total_found += passed_counts[x] + try_harder_counts[x];
            total_must_pass +=
                test_rxing_result.get_must_pass_count() + test_rxing_result.get_try_harder_count();
        }

        let total_tests = number_of_tests * test_count * 2;
        log::info(format!(
            "Decoded {} images out of {} ({}%, {} required)",
            total_found,
            total_tests,
            total_found * 100 / total_tests,
            total_must_pass
        ));

        match total_found.cmp(&(total_must_pass as usize)) {
            std::cmp::Ordering::Less => log::warning(format!(
                "--- Test failed by {} images",
                total_must_pass as usize - total_found
            )),
            std::cmp::Ordering::Equal => { /* totally fine */ }
            std::cmp::Ordering::Greater => log::warning(format!(
                "+++ Test too lax by {} images",
                total_found - total_must_pass as usize
            )),
        }

        // Then run through again and assert if any failed
        for x in 0..test_count {
            // for (int x = 0; x < testCount; x++) {
            let test_rxing_result = self.test_rxing_results.get(x).expect("ok");
            //   let label = "Rotation " + testRXingResult.getRotation() + " degrees: Too many images failed";
            assert!(
                passed_counts[x] >= test_rxing_result.get_must_pass_count() as usize,
                "Rotation {} degrees: Too many images failed",
                test_rxing_result.get_rotation()
            );
            assert!(
                try_harder_counts[x] >= test_rxing_result.get_try_harder_count() as usize,
                "Try harder, Rotation {} degrees: Too many images failed",
                test_rxing_result.get_rotation()
            );
        }
    }

    pub fn test_black_box_old(&mut self) {
        assert!(!self.test_rxing_results.is_empty());

        let image_files = self.get_image_files();
        let test_count = self.test_rxing_results.len();

        let mut passed_counts = vec![0usize; test_count];
        let mut misread_counts = vec![0usize; test_count];
        let mut try_harder_counts = vec![0usize; test_count];
        let mut try_harder_misread_counts = vec![0usize; test_count];

        for test_image in &image_files {
            // for (Path testImage : imageFiles) {
            log::info(format!("Starting {}", test_image.to_string_lossy()));

            let image = image::open(test_image).unwrap(); //ImageIO.read(testImage.toFile());

            //let testImageFileName = testImage.getFileName().toString();
            let file_base_name = test_image.file_stem().unwrap();
            //let expectedTextFile = self.testBase.resolve(fileBaseName + ".txt");
            let mut expected_text_file = test_image.clone();
            expected_text_file.set_extension("txt");
            let expected_text = if expected_text_file.exists() {
                Self::read_file_as_string(expected_text_file)
            } else {
                let mut new_path = self.test_base.clone().to_path_buf();
                new_path.push(file_base_name);
                new_path.set_extension("bin");
                //expectedTextFile = testBase.resolve(fileBaseName + ".bin");
                assert!(new_path.exists());
                Self::read_file_as_string(new_path)
            }
            .unwrap();

            let mut expected_metadata_file: PathBuf = self.test_base.clone().to_path_buf();
            expected_metadata_file.push(format!("{}.metadata", file_base_name.to_str().unwrap()));
            expected_metadata_file.set_extension("txt");
            let expected_metadata_unfinished = if expected_metadata_file.exists() {
                java_properties::read(
                    std::fs::File::open(expected_metadata_file)
                        .expect("file exists, we already know that"),
                )
                .expect("valid java properties file")
                // try (BufferedReader reader = Files.newBufferedReader(expectedMetadataFile, StandardCharsets.UTF_8)) {
                //   expectedMetadata.load(reader);
                // }
            } else {
                HashMap::new()
            };
            let mut expected_metadata = HashMap::new();
            for (k, v) in expected_metadata_unfinished {
                let new_k = RXingResultMetadataType::from(k);
                let new_v = match new_k {
                    RXingResultMetadataType::OTHER => RXingResultMetadataValue::OTHER(v),
                    RXingResultMetadataType::ORIENTATION => {
                        RXingResultMetadataValue::Orientation(v.parse().unwrap_or_default())
                    }
                    RXingResultMetadataType::BYTE_SEGMENTS => {
                        RXingResultMetadataValue::ByteSegments(vec![v.into_bytes()])
                    }
                    RXingResultMetadataType::ERROR_CORRECTION_LEVEL => {
                        RXingResultMetadataValue::ErrorCorrectionLevel(v)
                    }
                    RXingResultMetadataType::ISSUE_NUMBER => {
                        RXingResultMetadataValue::IssueNumber(v.parse().unwrap_or_default())
                    }
                    RXingResultMetadataType::SUGGESTED_PRICE => {
                        RXingResultMetadataValue::SuggestedPrice(v)
                    }
                    RXingResultMetadataType::POSSIBLE_COUNTRY => {
                        RXingResultMetadataValue::PossibleCountry(v)
                    }
                    RXingResultMetadataType::UPC_EAN_EXTENSION => {
                        RXingResultMetadataValue::UpcEanExtension(v)
                    }
                    RXingResultMetadataType::PDF417_EXTRA_METADATA => {
                        RXingResultMetadataValue::Pdf417ExtraMetadata(Arc::new(
                            PDF417RXingResultMetadata::default(),
                        ))
                    }
                    RXingResultMetadataType::STRUCTURED_APPEND_SEQUENCE => {
                        RXingResultMetadataValue::StructuredAppendSequence(
                            v.parse().unwrap_or_default(),
                        )
                    }
                    RXingResultMetadataType::STRUCTURED_APPEND_PARITY => {
                        RXingResultMetadataValue::StructuredAppendParity(
                            v.parse().unwrap_or_default(),
                        )
                    }
                    RXingResultMetadataType::SYMBOLOGY_IDENTIFIER => {
                        RXingResultMetadataValue::SymbologyIdentifier(v)
                    }
                    RXingResultMetadataType::IS_MIRRORED => {
                        RXingResultMetadataValue::IsMirrored(v.parse().unwrap())
                    }
                    RXingResultMetadataType::CONTENT_TYPE => {
                        RXingResultMetadataValue::ContentType(v)
                    }
                    RXingResultMetadataType::IS_INVERTED => {
                        RXingResultMetadataValue::IsInverted(v.parse().unwrap())
                    }
                    RXingResultMetadataType::FILTERED_CLOSED => {
                        RXingResultMetadataValue::FilteredClosed(v.parse().unwrap())
                    }
                    RXingResultMetadataType::FILTERED_RESOLUTION => {
                        // RXingResultMetadataValue::FilteredResolution(v.parse().unwrap())
                        let arr: Box<[usize]> = v
                            .split(',')
                            .map(|str_source| str_source.parse::<usize>().unwrap_or_default())
                            .take(2)
                            .collect();
                        RXingResultMetadataValue::FilteredResolution((arr[0], arr[1]))
                    }
                };
                expected_metadata.insert(new_k, new_v);
            }

            for x in 0..test_count {
                // for (int x = 0; x < testCount; x++) {
                let rotation = self.test_rxing_results.get(x).unwrap().get_rotation();
                let rotated_image = Self::rotate_image(&image, rotation);
                let source = BufferedImageLuminanceSource::new(rotated_image);
                let mut bitmap = BinaryBitmap::new(HybridBinarizer::new(source));

                // if file_base_name == "15" {
                // let mut f = File::create("test_file_output.txt").unwrap();
                // write!(f,"{}", bitmap.getBlackMatrix().unwrap());
                // drop(f);
                // Self::rotate_image(&image, rotation).save("test_image.png").unwrap();
                // }

                if let Ok(decoded) = self.decode(
                    &mut bitmap,
                    rotation,
                    &expected_text,
                    &expected_metadata,
                    false,
                ) {
                    if decoded {
                        passed_counts[x] += 1;
                    } else {
                        misread_counts[x] += 1;
                    }
                } else {
                    log::fine(format!("could not read at rotation {rotation}"));
                }
                // try {
                //   if (decode(bitmap, rotation, expectedText, expectedMetadata, false)) {
                //     passedCounts[x]+=1;
                //   } else {
                //     misreadCounts[x]+=1;
                //   }
                // } catch (ReaderException ignored) {
                //   log::fine(format!("could not read at rotation {}", rotation));
                // }
                if let Ok(decoded) = self.decode(
                    &mut bitmap,
                    rotation,
                    &expected_text,
                    &expected_metadata,
                    true,
                ) {
                    if decoded {
                        try_harder_counts[x] += 1;
                    } else {
                        try_harder_misread_counts[x] += 1;
                    }
                } else {
                    log::fine(format!("could not read at rotation {rotation} w/TH"));
                }
                // try {
                //   if (decode(bitmap, rotation, expectedText, expectedMetadata, true)) {
                //     tryHarderCounts[x]+=1;
                //   } else {
                //     tryHarderMisreadCounts[x]+=1;
                //   }
                // } catch (ReaderException ignored) {
                //   log::fine(format!("could not read at rotation {} w/TH", rotation));
                // }
            }
        }

        // Print the results of all tests first
        let mut total_found = 0;
        let mut total_must_pass = 0;
        let mut total_misread = 0;
        let mut total_max_misread = 0;

        for (x, test_rxing_result) in self.test_rxing_results.iter().enumerate() {
            //0..self.test_rxing_results.len() {
            // for (int x = 0; x < testRXingResults.size(); x++) {
            // let test_rxing_result = self.test_rxing_results.get(x).unwrap();
            log::info(format!(
                "Rotation {} degrees:",
                test_rxing_result.get_rotation()
            ));
            log::info(format!(
                " {} of {} images passed ({} required)",
                passed_counts[x],
                image_files.len(),
                test_rxing_result.get_must_pass_count()
            ));
            let mut failed = image_files.len() - passed_counts[x];
            log::info(format!(
                " {} failed due to misreads, {} not detected",
                misread_counts[x],
                failed - misread_counts[x]
            ));
            log::info(format!(
                " {} of {} images passed with try harder ({} required)",
                try_harder_counts[x],
                image_files.len(),
                test_rxing_result.get_try_harder_count()
            ));
            failed = image_files.len() - try_harder_counts[x];
            log::info(format!(
                " {} failed due to misreads, {} not detected",
                try_harder_misread_counts[x],
                failed - try_harder_misread_counts[x]
            ));
            total_found += passed_counts[x] + try_harder_counts[x];
            total_must_pass +=
                test_rxing_result.get_must_pass_count() + test_rxing_result.get_try_harder_count();
            total_misread += misread_counts[x] + try_harder_misread_counts[x];
            total_max_misread += test_rxing_result.get_max_misreads()
                + test_rxing_result.get_max_try_harder_misreads();
        }

        let total_tests = image_files.len() * test_count * 2;
        log::info(format!(
            "Decoded {} images out of {} ({}, {} required)",
            total_found,
            total_tests,
            total_found * 100 / total_tests,
            total_must_pass
        ));

        match total_found.cmp(&(total_must_pass as usize)) {
            std::cmp::Ordering::Less => log::warning(format!(
                "--- Test failed by {} images",
                total_must_pass as usize - total_found
            )),
            std::cmp::Ordering::Equal => { /* totally ok */ }
            std::cmp::Ordering::Greater => log::warning(format!(
                "+++ Test too lax by {} images",
                total_found - total_must_pass as usize
            )),
        }

        match total_misread.cmp(&(total_max_misread as usize)) {
            std::cmp::Ordering::Less => log::warning(format!(
                "+++ Test expects too many misreads by {} images",
                total_max_misread as usize - total_misread
            )),
            std::cmp::Ordering::Equal => { /* this is fine */ }
            std::cmp::Ordering::Greater => log::warning(format!(
                "--- Test had too many misreads by {} images",
                total_misread - total_max_misread as usize
            )),
        }

        // Then run through again and assert if any failed
        for x in 0..test_count {
            // for (int x = 0; x < testCount; x++) {
            let test_rxing_result = self.test_rxing_results.get(x).unwrap();
            let label = format!(
                "Rotation {} degrees: Too many images failed",
                test_rxing_result.get_rotation()
            );
            assert!(
                passed_counts[x] >= test_rxing_result.get_must_pass_count() as usize,
                "{}",
                label
            );
            assert!(
                try_harder_counts[x] >= test_rxing_result.get_try_harder_count() as usize,
                "Try harder, {label}",
            );
            let label = format!(
                "Rotation {} degrees: Too many images misread",
                test_rxing_result.get_rotation()
            );
            assert!(
                misread_counts[x] <= test_rxing_result.get_max_misreads() as usize,
                "{}",
                label
            );
            assert!(
                try_harder_misread_counts[x]
                    <= test_rxing_result.get_max_try_harder_misreads() as usize,
                "Try harder, {label}"
            );
        }
    }

    fn decode<B: Binarizer>(
        &mut self,
        source: &mut BinaryBitmap<B>,
        rotation: f32,
        expected_text: &str,
        expected_metadata: &HashMap<RXingResultMetadataType, RXingResultMetadataValue>,
        try_harder: bool,
    ) -> Result<bool> {
        let suffix = format!(
            " ({}rotation: {})",
            if try_harder { "try harder, " } else { "" },
            rotation
        );

        let mut hints = self.hints.clone();
        if try_harder {
            hints.TryHarder = Some(true);
            // hints.put(DecodeHintType.TRY_HARDER, Boolean.TRUE);
        }

        // Try in 'pure' mode mostly to exercise PURE_BARCODE code paths for exceptions;
        // not expected to pass, generally
        // let mut result = None;
        let pure_hints = DecodeHints::default().with(DecodeHintValue::PureBarcode(true));
        let mut result = self
            .barcode_reader
            .decode_with_hints(source, &pure_hints)
            .ok();

        if result.is_none() {
            result = Some(self.barcode_reader.decode_with_hints(source, &hints)?)
        }

        let result = result.unwrap();

        if &self.expected_format != result.getBarcodeFormat() {
            log::info(format!(
                "Format mismatch: expected '{:?}' but got '{:?}'{}",
                self.expected_format,
                result.getBarcodeFormat(),
                suffix
            ));
            return Ok(false);
        }

        let result_text = result.getText();
        if expected_text != result_text {
            log::info(format!(
                "Content mismatch: expected '{expected_text}' but got '{result_text}'{suffix}"
            ));
            return Ok(false);
        }

        let result_metadata = result.getRXingResultMetadata();
        for (key, value) in expected_metadata {
            // let key = RXingResultMetadataType.valueOf(metadatum.getKey().toString());
            // Object expectedValue = metadatum.getValue();
            let actual_value = result_metadata.get(key);
            if actual_value.is_none() || !(value == actual_value.unwrap()) {
                log::info(format!(
                    "Metadata mismatch for key '{key:?}': expected '{value:?}' but got '{actual_value:?}'"
                ));
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn read_file_as_string(file: PathBuf) -> Result<String, std::io::Error> {
        let string_contents = if let Some(ext) = file.extension() {
            if ext == "bin" {
                let mut buffer: Vec<u8> = Vec::new();
                File::open(&file)?.read_to_end(&mut buffer)?;
                CharacterSet::ISO8859_1
                    .decode_replace(&buffer)
                    .expect("decode")
            } else {
                read_to_string(&file).expect("ok")
            }
        } else {
            String::default()
        };
        // let string_contents = read_to_string(&file)?; //new String(Files.readAllBytes(file), charset);
        if string_contents.ends_with('\n') {
            log::info(format!("String contents of file {} end with a newline. This may not be intended and cause a test failure",file.to_string_lossy()));
        }
        Ok(string_contents)
    }

    fn rotate_image(original: &image::DynamicImage, degrees: f32) -> image::DynamicImage {
        if degrees == 0.0 {
            return original.clone();
        }

        // switch (original.getType()) {
        //   case BufferedImage.TYPE_BYTE_INDEXED:
        //   case BufferedImage.TYPE_BYTE_BINARY:
        //     BufferedImage argb = new BufferedImage(original.getWidth(),
        //                                            original.getHeight(),
        //                                            BufferedImage.TYPE_INT_ARGB);
        //     Graphics g = argb.createGraphics();
        //     g.drawImage(original, 0, 0, null);
        //     g.dispose();
        //     original = argb;
        //     break;
        // }

        if degrees == 90.0 {
            original.rotate90()
        } else if degrees == 180.0 {
            original.rotate180()
        } else if degrees == 270.0 {
            original.rotate270()
        } else {
            let radians = degrees.to_radians();

            {
                use image::DynamicImage;
                use image::Luma;
                use imageproc::geometric_transformations::*;

                let i = rotate_about_center(
                    &original.to_luma8(),
                    radians,
                    Interpolation::Nearest,
                    Luma([u8::MAX / 2; 1]),
                );

                DynamicImage::from(i)
            }
        }

        // let radians = degrees.to_radians();

        // {
        //     use image::DynamicImage;
        //     use image::Luma;
        //     use imageproc::geometric_transformations::*;

        //     let i = rotate_about_center(
        //         &original.to_luma8(),
        //         radians,
        //         Interpolation::Nearest,
        //         Luma([u8::MAX / 2; 1]),
        //     );

        //     DynamicImage::from(i)
        // }

        // // Transform simply to find out the new bounding box (don't actually run the image through it)
        // AffineTransform at = new AffineTransform();
        // at.rotate(radians, original.getWidth() / 2.0, original.getHeight() / 2.0);
        // BufferedImageOp op = new AffineTransformOp(at, AffineTransformOp.TYPE_BICUBIC);

        // RectangularShape r = op.getBounds2D(original);
        // int width = (int) Math.ceil(r.getWidth());
        // int height = (int) Math.ceil(r.getHeight());

        // // Real transform, now that we know the size of the new image and how to translate after we rotate
        // // to keep it centered
        // at = new AffineTransform();
        // at.rotate(radians, width / 2.0, height / 2.0);
        // at.translate((width - original.getWidth()) / 2.0,
        //              (height - original.getHeight()) / 2.0);
        // op = new AffineTransformOp(at, AffineTransformOp.TYPE_BICUBIC);

        // return op.filter(original, new BufferedImage(width, height, original.getType()));
    }

    fn get_meta(result: &RXingResult) -> Option<Arc<PDF417RXingResultMetadata>> {
        if let Some(RXingResultMetadataValue::Pdf417ExtraMetadata(mtd)) = result
            .getRXingResultMetadata()
            .get(&RXingResultMetadataType::PDF417_EXTRA_METADATA)
        {
            /*(PDF417RXingResultMetadata)*/
            Some(mtd.clone())
        } else {
            None
        }
    }

    fn decode_pdf417<B: Binarizer>(
        source: &mut BinaryBitmap<B>,
        try_harder: bool,
        barcode_reader: &mut T,
    ) -> Result<Vec<RXingResult>> {
        let hints = DecodeHints::default().with(DecodeHintValue::TryHarder(try_harder));

        barcode_reader.decode_multiple_with_hints(source, &hints)
    }

    fn get_image_file_lists(&self) -> Result<HashMap<String, Vec<PathBuf>>, std::io::Error> {
        let mut result: HashMap<String, Vec<PathBuf>> = HashMap::new();
        for file in self.get_image_files() {
            let test_image_file_name = file
                .file_name()
                .expect("file name exists")
                .to_str()
                .unwrap();
            let file_base_name = if let Some(pos) = test_image_file_name.find('-') {
                &test_image_file_name[..pos]
            } else {
                test_image_file_name
            };
            if !result.contains_key(file_base_name) {
                result.insert(file_base_name.to_owned(), Vec::new());
            }
            result.get_mut(file_base_name).as_mut().unwrap().push(file);

            //   String fileBaseName = testImageFileName.substring(0, testImageFileName.indexOf('-'));
            //let files = result.computeIfAbsent(fileBaseName, k -> new ArrayList<>());
            //   files.add(file);
            // result.insert(fileBaseName, vec![]);
        }

        Ok(result)
    }
}

mod log {
    pub fn info(data: String) {
        prn("INFO", data)
    }

    pub fn fine(data: String) {
        prn("FINE", data)
    }

    pub fn warning(data: String) {
        prn("WARN", data)
    }

    fn prn(level: &str, data: String) {
        println!("{level} :: {data}")
    }
}

// dxfilmedge-1

#![cfg(feature = "image")]

use rxing::{BarcodeFormat, MultiFormatReader};

mod common;

/**
 * @author Sean Owen
 */
#[cfg(feature = "image-formats")]
#[test]
fn dx_film_edge() {
    let mut tester = common::AbstractBlackBoxTestCase::new(
        "test_resources/blackbox/dxfilmedge-1",
        MultiFormatReader::default(),
        BarcodeFormat::DXFilmEdge,
    );

    tester.add_test(1, 2, 0.0);
    tester.add_test(1, 2, 90.0);
    tester.add_test(1, 2, 180.0);
    tester.add_test(1, 2, 320.0);

    tester.test_black_box()
}

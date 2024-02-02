use crate::common::Result;
use crate::LuminanceSource;

/// A simple luma8 source for bytes, supports cropping but not rotation
#[derive(Debug, Clone)]
pub struct Luma8LuminanceSource {
    /// image dimension in form (x,y)
    dimensions: (u32, u32),
    /// image origin in the form (x,y)
    origin: (u32, u32),
    /// raw data for luma 8
    data: Vec<u8>,
    /// flag indicating if the underlying data needs to be inverted for use
    inverted: bool,
    /// original dimensions of the data, used to manage crop
    original_dimension: (u32, u32),
}
impl LuminanceSource for Luma8LuminanceSource {
    fn get_row(&self, y: usize) -> Vec<u8> {
        let chunk_size = self.original_dimension.0 as usize;
        let row_skip = y + self.origin.1 as usize;
        let column_skip = self.origin.0 as usize;
        let column_take = self.dimensions.0 as usize;

        let data_start = (chunk_size * row_skip) + column_skip;
        let data_end = (chunk_size * row_skip) + column_skip + column_take;

        if self.inverted {
            self.invert_block_of_bytes(Vec::from(&self.data[data_start..data_end]))
        } else {
            Vec::from(&self.data[data_start..data_end])
        }

        // self.data
        //     .chunks_exact(chunk_size)
        //     .skip(row_skip)
        //     .take(1)
        //     .flatten()
        //     .skip(column_skip)
        //     .take(column_take)
        //     .map(|byte| Self::invert_if_should(*byte, self.inverted))
        //     .collect()
    }

    fn get_column(&self, x: usize) -> Vec<u8> {
        self.data
            .chunks_exact(self.original_dimension.0 as usize)
            .skip(self.origin.1 as usize)
            .fold(Vec::default(), |mut acc, e| {
                acc.push(e[self.origin.0 as usize + x as usize]);
                acc
            })
            .iter()
            .map(|byte| Self::invert_if_should(*byte, self.inverted))
            .collect()
    }

    fn get_matrix(&self) -> Vec<u8> {
        self.data
            .iter()
            .skip((self.original_dimension.0 * self.origin.1) as usize)
            .take((self.dimensions.1 * self.original_dimension.0) as usize)
            .collect::<Vec<&u8>>()
            .chunks_exact(self.original_dimension.0 as usize)
            .flat_map(|f| {
                f.iter()
                    .skip((self.origin.0) as usize)
                    .take(self.get_width())
                    .copied()
            }) // flatten this all out
            .copied() // copy it over so that it's u8
            .map(|byte| Self::invert_if_should(byte, self.inverted))
            .collect() // collect into a vec
    }

    fn get_width(&self) -> usize {
        self.dimensions.0 as usize
    }

    fn get_height(&self) -> usize {
        self.dimensions.1 as usize
    }

    fn invert(&mut self) {
        self.inverted = !self.inverted;
    }

    fn is_crop_supported(&self) -> bool {
        true
    }

    fn crop(&self, left: usize, top: usize, width: usize, height: usize) -> Result<Self> {
        Ok(Self {
            dimensions: (width as u32, height as u32),
            origin: (self.origin.0 + left as u32, self.origin.1 + top as u32),
            data: self.data.clone(),
            inverted: self.inverted,
            original_dimension: self.original_dimension,
        })
    }

    fn is_rotate_supported(&self) -> bool {
        true
    }

    fn rotate_counter_clockwise(&self) -> Result<Self> {
        let mut new_matrix = Self {
            dimensions: self.dimensions,
            origin: self.origin,
            data: self.data.clone(),
            inverted: self.inverted,
            original_dimension: self.original_dimension,
        };
        new_matrix.transpose();
        new_matrix.reverseColumns();
        Ok(new_matrix)
    }

    fn rotate_counter_clockwise_45(&self) -> Result<Self> {
        Err(crate::Exceptions::unsupported_operation_with(
            "This luminance source does not support rotation by 45 degrees.",
        ))
    }

    fn get_luma8_point(&self, column: usize, row: usize) -> u8 {
        self.get_row(row)[column]
    }
}

impl Luma8LuminanceSource {
    fn reverseColumns(&mut self) {
        for col in 0..(self.get_width()) {
            let mut a = 0;
            let mut b = self.get_height() - 1;
            while a < b {
                let offset_a = a * self.get_width() + col;
                let offset_b = b * self.get_width() + col;
                self.data.swap(offset_a, offset_b);

                a += 1;
                b -= 1;
            }
        }
    }

    fn transpose_square(&mut self) {
        for i in 0..self.get_height() {
            for j in i..self.get_width() {
                let offset_a = (self.get_width() * i) + j;
                let offset_b = (self.get_width() * j) + i;
                self.data.swap(offset_a, offset_b);
            }
        }
    }

    fn transpose_rect(&mut self) {
        let mut new_data = vec![0; self.data.len()];
        let new_dim = (self.dimensions.1, self.dimensions.0);
        for i in 0..self.get_height() {
            for j in 0..self.get_width() {
                let offset_a = (self.get_width() * i) + j;
                let offset_b = (self.get_height() * j) + i;
                new_data[offset_b] = self.data[offset_a];
            }
        }
        self.data = new_data;
        self.dimensions = new_dim;
        self.original_dimension = (self.original_dimension.1, self.original_dimension.0);
        self.origin = (self.origin.1, self.origin.0);
    }

    fn transpose(&mut self) {
        if self.get_width() == self.get_height() {
            self.transpose_square()
        } else {
            self.transpose_rect()
        }
        // print_matrix(&self.data, self.get_width(), self.get_height());
    }
}

impl Luma8LuminanceSource {
    pub fn new(source: Vec<u8>, width: u32, height: u32) -> Self {
        Self {
            dimensions: (width, height),
            origin: (0, 0),
            data: source,
            inverted: false,
            original_dimension: (width, height),
        }
    }

    pub fn with_empty_image(width: usize, height: usize) -> Self {
        Self {
            dimensions: (width as u32, height as u32),
            origin: (0, 0),
            data: vec![0u8; width * height],
            inverted: false,
            original_dimension: (width as u32, height as u32),
        }
    }

    pub fn get_matrix_mut(&mut self) -> &mut Vec<u8> {
        &mut self.data
    }

    #[inline(always)]
    fn invert_if_should(byte: u8, invert: bool) -> u8 {
        if invert {
            255 - byte
        } else {
            byte
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Luma8LuminanceSource, LuminanceSource};

    #[test]
    fn test_rotate() {
        let src_square = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];

        let src_rect = vec![0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 0, 0];

        let square = Luma8LuminanceSource::new(src_square, 3, 3);
        let rect_tall = Luma8LuminanceSource::new(src_rect.clone(), 3, 4);
        let rect_wide = Luma8LuminanceSource::new(src_rect, 4, 3);

        let rotated_square = square.rotate_counter_clockwise().expect("rotate");
        // print_matrix(&src_rect, 4, 3);
        let rotated_wide_rect = rect_wide.rotate_counter_clockwise().expect("rotate");
        // print_matrix(&src_rect, 3, 4);
        let rotated_tall_rect = rect_tall.rotate_counter_clockwise().expect("rotate");

        assert_eq!(rotated_square.dimensions, square.dimensions);
        assert_eq!(
            rotated_tall_rect.dimensions,
            (rect_tall.dimensions.1, rect_tall.dimensions.0)
        );
        assert_eq!(
            rotated_wide_rect.dimensions,
            (rect_wide.dimensions.1, rect_wide.dimensions.0)
        );

        assert_eq!(rotated_square.data, vec![3, 6, 9, 2, 5, 8, 1, 4, 7]);

        assert_eq!(
            rotated_wide_rect.data,
            vec![1, 1, 0, 0, 1, 0, 1, 1, 0, 0, 0, 1]
        );

        assert_eq!(
            rotated_tall_rect.data,
            vec![0, 1, 1, 0, 1, 0, 1, 0, 0, 1, 1, 0]
        );
    }
}

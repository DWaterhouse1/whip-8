use grid::Grid;
use strum_macros::Display;

#[derive(Debug, Display, PartialEq, Eq, Clone, Copy)]
pub enum Pixel {
    Off,
    On,
}

impl Pixel {
    fn flip(&mut self) -> bool {
        match self {
            Pixel::Off => {
                *self = Pixel::On;
                false
            }
            Pixel::On => {
                *self = Pixel::Off;
                true
            }
        }
    }
}

#[derive(Debug, Display, PartialEq, Eq, Clone, Copy)]
pub enum PixelsDisabled {
    NoPixels,
    SomePixels,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Display {
    display_buffer: Grid<Pixel>,
    dirty: bool,
}

impl Display {
    pub fn new(width: usize, height: usize) -> Self {
        Display {
            display_buffer: Grid::<Pixel>::init(height, width, Pixel::Off),
            dirty: true,
        }
    }

    pub fn from_vec(vec: Vec<Pixel>, cols: usize) -> Self {
        Display {
            display_buffer: Grid::<Pixel>::from_vec(vec, cols),
            dirty: true,
        }
    }

    pub fn clear(&mut self) {
        self.display_buffer.fill(Pixel::Off);
        self.dirty = true;
    }

    pub fn draw_sprite(&mut self, x: usize, y: usize, data: &[u8]) -> PixelsDisabled {
        let leftmost_column = x % self.display_buffer.cols();
        let mut row = y % self.display_buffer.rows();
        let mut pixels_disabled = PixelsDisabled::NoPixels;

        for datum in data {
            if row >= self.display_buffer.rows() {
                break;
            }

            if self.draw_byte(leftmost_column, row, *datum) == PixelsDisabled::SomePixels {
                pixels_disabled = PixelsDisabled::SomePixels;
            }

            row += 1;
        }

        self.dirty = true;
        pixels_disabled
    }

    pub fn get_display_buffer(&mut self) -> Option<&Grid<Pixel>> {
        if self.dirty {
            self.dirty = false;
            Some(&self.display_buffer)
        } else {
            None
        }
    }

    fn draw_byte(&mut self, col: usize, row: usize, value: u8) -> PixelsDisabled {
        let mut draw_column = col;
        let mut turned_any_off = false;

        for shift in 0..8 {
            match self.display_buffer.get_mut(row, draw_column) {
                Some(pixel) => {
                    if (value >> (7 - shift)) & 1 == 1 {
                        turned_any_off |= pixel.flip();
                    }
                    draw_column += 1;
                }
                None => break,
            }
        }

        if turned_any_off {
            PixelsDisabled::SomePixels
        } else {
            PixelsDisabled::NoPixels
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_display_all_off() {
        let display = Display::new(8, 8);
        for pixel in display.display_buffer.iter() {
            assert_eq!(*pixel, Pixel::Off);
        }
    }

    #[test]
    fn test_create_display_all_on() {
        let display = Display::from_vec(vec![Pixel::On; 64], 8);
        for pixel in display.display_buffer.iter() {
            assert_eq!(*pixel, Pixel::On);
        }
    }

    #[test]
    fn test_clear() {
        let mut display = Display::from_vec(vec![Pixel::On; 64], 8);
        display.clear();
        for pixel in display.display_buffer.iter() {
            assert_eq!(*pixel, Pixel::Off);
        }
    }

    #[test]
    fn test_draw_solid_row() {
        let mut display = Display::new(8, 8);
        display.draw_sprite(0, 0, &[0xFF]);

        #[rustfmt::skip]
        let expected = Display::from_vec(
            vec![
                Pixel::On,  Pixel::On,  Pixel::On,  Pixel::On,  Pixel::On,  Pixel::On,  Pixel::On,  Pixel::On,
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
            ],
            8,
        );

        assert_eq!(display, expected);
    }

    #[test]
    fn test_draw_rightside_oob() {
        let mut display = Display::new(8, 8);
        display.draw_sprite(4, 0, &[0xFF]);

        #[rustfmt::skip]
        let expected = Display::from_vec(
            vec![
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::On,  Pixel::On,  Pixel::On,  Pixel::On,
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
            ],
            8,
        );

        assert_eq!(display, expected);
    }

    #[test]
    fn test_draw_lower_oob() {
        let mut display = Display::new(8, 8);
        display.draw_sprite(0, 6, &[0xFF, 0xFF, 0xAB, 0xCD]);

        #[rustfmt::skip]
        let expected = Display::from_vec(
            vec![
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
                Pixel::On , Pixel::On , Pixel::On , Pixel::On , Pixel::On,  Pixel::On,  Pixel::On,  Pixel::On,
                Pixel::On , Pixel::On , Pixel::On , Pixel::On , Pixel::On,  Pixel::On,  Pixel::On,  Pixel::On,
            ],
            8,
        );

        assert_eq!(display, expected);
    }

    #[test]
    fn test_draw_wrapped() {
        let mut display = Display::new(8, 8);
        display.draw_sprite(12, 9, &[0xFF]);

        #[rustfmt::skip]
        let expected = Display::from_vec(
            vec![
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::On,  Pixel::On,  Pixel::On,  Pixel::On,
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
            ],
            8,
        );

        assert_eq!(display, expected);
    }

    #[test]
    fn test_draw_multiple_rows() {
        let mut display = Display::new(8, 8);
        display.draw_sprite(0, 0, &[0x0F, 0xF0]);

        #[rustfmt::skip]
        let expected = Display::from_vec(
            vec![
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::On,  Pixel::On,  Pixel::On,  Pixel::On,
                Pixel::On , Pixel::On , Pixel::On , Pixel::On , Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
            ],
            8,
        );

        assert_eq!(display, expected);
    }

    #[test]
    fn test_draw_multiple_sprites() {
        let mut display = Display::new(8, 8);
        assert_eq!(display.draw_sprite(0, 0, &[0x0F]), PixelsDisabled::NoPixels);
        assert_eq!(display.draw_sprite(0, 1, &[0xF0]), PixelsDisabled::NoPixels);

        #[rustfmt::skip]
        let expected = Display::from_vec(
            vec![
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::On,  Pixel::On,  Pixel::On,  Pixel::On,
                Pixel::On , Pixel::On , Pixel::On , Pixel::On , Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
            ],
            8,
        );

        assert_eq!(display, expected);
    }

    #[test]
    fn test_draw_overlapping_sprites() {
        let mut display = Display::new(8, 8);
        assert_eq!(
            display.draw_sprite(0, 3, &[0xFF, 0xFF]),
            PixelsDisabled::NoPixels,
        );
        assert_eq!(
            display.draw_sprite(0, 3, &[0xF0, 0x0F]),
            PixelsDisabled::SomePixels
        );

        #[rustfmt::skip]
        let expected = Display::from_vec(
            vec![
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::On,  Pixel::On,  Pixel::On,  Pixel::On,
                Pixel::On , Pixel::On , Pixel::On , Pixel::On , Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
                Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off, Pixel::Off,
            ],
            8,
        );

        assert_eq!(display, expected);
    }
}

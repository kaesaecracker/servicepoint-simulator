use crate::static_font;
use font_kit::canvas::{Canvas, Format, RasterizationOptions};
use font_kit::font::Font;
use font_kit::hinting::HintingOptions;
use pathfinder_geometry::transform2d::Transform2F;
use pathfinder_geometry::vector::{vec2f, vec2i};
use servicepoint::{Bitmap, Grid, TILE_SIZE};

const DEFAULT_FONT_FILE: &[u8] = include_bytes!("../Web437_IBM_BIOS.woff");

const CHAR_COUNT: usize = u8::MAX as usize + 1;

pub struct BitmapFont {
    bitmaps: [Bitmap; CHAR_COUNT],
}

impl BitmapFont {
    pub fn new(bitmaps: [Bitmap; CHAR_COUNT]) -> Self {
        Self { bitmaps }
    }

    pub fn load(font: Font, size: usize) -> BitmapFont {
        let mut bitmaps =
            core::array::from_fn(|_| Bitmap::new(TILE_SIZE, TILE_SIZE));
        let mut canvas =
            Canvas::new(vec2i(size as i32, size as i32), Format::A8);
        let size_f = size as f32;
        let transform = Transform2F::default();

        for char_code in u8::MIN..=u8::MAX {
            let char = char_code as char;
            let glyph_id = match font.glyph_for_char(char) {
                None => continue,
                Some(val) => val,
            };

            canvas.pixels.fill(0);
            font.rasterize_glyph(
                &mut canvas,
                glyph_id,
                size_f,
                Transform2F::from_translation(vec2f(0f32, size_f)) * transform,
                HintingOptions::None,
                RasterizationOptions::GrayscaleAa,
            )
            .unwrap();

            assert_eq!(canvas.pixels.len(), size * size);
            assert_eq!(canvas.stride, size);

            let bitmap = &mut bitmaps[char_code as usize];
            for y in 0..TILE_SIZE {
                for x in 0..TILE_SIZE {
                    let index = x + y * TILE_SIZE;
                    let canvas_val = canvas.pixels[index] != 0;
                    bitmap.set(x, y, canvas_val);
                }
            }
        }

        Self::new(bitmaps)
    }

    pub fn get_bitmap(&self, char_code: u8) -> &Bitmap {
        &self.bitmaps[char_code as usize]
    }
}

impl Default for BitmapFont {
    fn default() -> Self {
        static_font::load_static()
    }
}

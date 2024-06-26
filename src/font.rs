use std::sync::Arc;

use font_kit::canvas::{Canvas, Format, RasterizationOptions};
use font_kit::font::Font;
use font_kit::hinting::HintingOptions;
use pathfinder_geometry::transform2d::Transform2F;
use pathfinder_geometry::vector::{vec2f, vec2i};
use servicepoint::{Grid, PixelGrid, TILE_SIZE};

const DEFAULT_FONT_FILE: &[u8] = include_bytes!("../Web437_IBM_BIOS.woff");

pub struct BitmapFont {
    bitmaps: [PixelGrid; u8::MAX as usize],
}

impl BitmapFont {
    pub fn load(font: Font) -> BitmapFont {
        let mut bitmaps =
            core::array::from_fn(|_| PixelGrid::new(TILE_SIZE, TILE_SIZE));

        for char_code in u8::MIN..u8::MAX {
            let char = char_code as char;
            let glyph_id = match font.glyph_for_char(char) {
                None => continue,
                Some(val) => val,
            };

            let size = 8f32;
            let transform = Transform2F::default();
            let mut canvas =
                Canvas::new(vec2i(size as i32, size as i32), Format::A8);
            font.rasterize_glyph(
                &mut canvas,
                glyph_id,
                size,
                Transform2F::from_translation(vec2f(0f32, size)) * transform,
                HintingOptions::None,
                RasterizationOptions::GrayscaleAa,
            )
            .unwrap();

            assert_eq!(canvas.pixels.len(), 64);
            assert_eq!(canvas.stride, 8);

            for y in 0..TILE_SIZE {
                for x in 0..TILE_SIZE {
                    let index = x + y * TILE_SIZE;
                    let canvas_val = canvas.pixels[index] != 0;
                    bitmaps[char_code as usize].set(x, y, canvas_val);
                }
            }
        }

        BitmapFont { bitmaps }
    }

    pub fn get_bitmap(&self, char_code: u8) -> &PixelGrid {
        &self.bitmaps[char_code as usize]
    }
}

impl Default for BitmapFont {
    fn default() -> Self {
        let font = Font::from_bytes(Arc::new(DEFAULT_FONT_FILE.to_vec()), 0)
            .expect("could not load included font");
        Self::load(font)
    }
}

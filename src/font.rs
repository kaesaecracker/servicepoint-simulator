use font_kit::canvas::*;
use font_kit::hinting::HintingOptions;
use pathfinder_geometry::transform2d::Transform2F;
use pathfinder_geometry::vector::{vec2f, vec2i};
use servicepoint2::{PixelGrid, TILE_SIZE};

pub struct BitmapFont {
    bitmaps: [PixelGrid; u8::MAX as usize],
}

impl BitmapFont {
    pub fn load_file(file: &str) -> BitmapFont {
        let font = font_kit::font::Font::from_path(file, 0)
            .expect("could not load font");

        let mut bitmaps = core::array::from_fn(|_| {
            PixelGrid::new(TILE_SIZE as usize, TILE_SIZE as usize)
        });

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

            for y in 0..TILE_SIZE as usize {
                for x in 0..TILE_SIZE as usize {
                    let index = x + y * TILE_SIZE as usize;
                    let canvas_val = canvas.pixels[index] != 0;
                    bitmaps[char_code as usize].set(x, y, canvas_val);
                }
            }
        }

        return BitmapFont { bitmaps };
    }

    pub fn get_bitmap(&self, char_code: u8) -> &PixelGrid {
        &self.bitmaps[char_code as usize]
    }
}

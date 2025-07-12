use crate::font_renderer::RenderError::{GlyphNotFound, OutOfBounds};
use font_kit::{
    canvas::{Canvas, Format, RasterizationOptions},
    error::GlyphLoadingError,
    family_name::FamilyName,
    font::Font,
    hinting::HintingOptions,
    properties::Properties,
    source::SystemSource,
};
use pathfinder_geometry::{
    transform2d::Transform2F,
    vector::{vec2f, vec2i},
};
use servicepoint::{Bitmap, GridMut, WindowMut, TILE_SIZE};
use std::{
    collections::HashMap,
    sync::{Mutex, MutexGuard},
};

#[derive(Debug)]
struct SendFont(Font);

// struct is only using primitives and pointers - lets try if it is only missing the declaration
unsafe impl Send for SendFont {}

impl AsRef<Font> for SendFont {
    fn as_ref(&self) -> &Font {
        &self.0
    }
}

#[derive(Debug)]
pub struct FontRenderer8x8 {
    font: SendFont,
    canvas: Mutex<Canvas>,
    fallback_char: Option<u32>,
    cache: Mutex<HashMap<char, Bitmap>>,
}

#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("Glyph not found for '{0}'")]
    GlyphNotFound(char),
    #[error(transparent)]
    GlyphLoadingError(#[from] GlyphLoadingError),
    #[error("out of bounds at {0} {1}")]
    OutOfBounds(usize, usize),
}

impl FontRenderer8x8 {
    const FALLBACK_CHAR: char = '?';
    pub fn new(font: Font) -> Self {
        let canvas =
            Canvas::new(vec2i(TILE_SIZE as i32, TILE_SIZE as i32), Format::A8);
        assert_eq!(canvas.pixels.len(), TILE_SIZE * TILE_SIZE);
        assert_eq!(canvas.stride, TILE_SIZE);
        let fallback_char = font.glyph_for_char(Self::FALLBACK_CHAR);
        Self {
            font: SendFont(font),
            fallback_char,
            canvas: Mutex::new(canvas),
            cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn from_name(family_name: String) -> Self {
        let font = SystemSource::new()
            .select_best_match(
                &[FamilyName::Title(family_name)],
                &Properties::new(),
            )
            .unwrap()
            .load()
            .unwrap();
        Self::new(font)
    }

    pub fn render(
        &self,
        char: char,
        target: &mut WindowMut<bool, Bitmap>,
    ) -> Result<(), RenderError> {
        let cache = &mut *self.cache.lock().unwrap();
        if let Some(drawn_char) = cache.get(&char) {
            target.deref_assign(drawn_char);
        }

        let glyph_id = self.get_glyph(char)?;

        let mut canvas = self.canvas.lock().unwrap();
        canvas.pixels.fill(0);
        self.font.as_ref().rasterize_glyph(
            &mut canvas,
            glyph_id,
            TILE_SIZE as f32,
            Transform2F::from_translation(vec2f(0f32, TILE_SIZE as f32))
                * Transform2F::default(),
            HintingOptions::None,
            RasterizationOptions::Bilevel,
        )?;

        let mut bitmap = Bitmap::new(TILE_SIZE, TILE_SIZE).unwrap();
        Self::copy_to_bitmap(canvas, &mut bitmap)?;
        target.deref_assign(&bitmap);
        cache.insert(char, bitmap);
        Ok(())
    }

    fn copy_to_bitmap(
        canvas: MutexGuard<Canvas>,
        bitmap: &mut Bitmap,
    ) -> Result<(), RenderError> {
        for y in 0..TILE_SIZE {
            for x in 0..TILE_SIZE {
                let canvas_val = canvas.pixels[x + y * TILE_SIZE] != 0;
                if !bitmap.set_optional(x, y, canvas_val) {
                    return Err(OutOfBounds(x, y));
                }
            }
        }
        Ok(())
    }

    fn get_glyph(&self, char: char) -> Result<u32, RenderError> {
        self.font
            .as_ref()
            .glyph_for_char(char)
            .or(self.fallback_char)
            .ok_or(GlyphNotFound(char))
    }
}

impl Default for FontRenderer8x8 {
    fn default() -> Self {
        let utf8_font = SystemSource::new()
            .select_best_match(&[FamilyName::Monospace], &Properties::new())
            .unwrap()
            .load()
            .unwrap();
        FontRenderer8x8::new(utf8_font)
    }
}

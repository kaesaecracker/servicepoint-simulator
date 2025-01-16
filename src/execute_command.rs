use log::{debug, error, info, trace, warn};
use servicepoint::{
    Bitmap, BrightnessGrid, CharGrid, Command, Cp437Grid, Grid, Origin, Tiles,
    PIXEL_COUNT, PIXEL_WIDTH, TILE_SIZE,
};
use std::sync::{RwLock, RwLockWriteGuard};

use crate::font::Cp437Font;
use crate::font_renderer::FontRenderer8x8;

pub(crate) fn execute_command(
    command: Command,
    cp436_font: &Cp437Font,
    utf8_font: &FontRenderer8x8,
    display_ref: &RwLock<Bitmap>,
    luma_ref: &RwLock<BrightnessGrid>,
) -> bool {
    debug!("received {command:?}");
    match command {
        Command::Clear => {
            info!("clearing display");
            display_ref.write().unwrap().fill(false);
        }
        Command::HardReset => {
            warn!("display shutting down");
            return false;
        }
        Command::BitmapLinearWin(Origin { x, y, .. }, pixels, _) => {
            let mut display = display_ref.write().unwrap();
            print_pixel_grid(x, y, &pixels, &mut display);
        }
        Command::Cp437Data(origin, grid) => {
            let mut display = display_ref.write().unwrap();
            print_cp437_data(origin, &grid, cp436_font, &mut display);
        }
        #[allow(deprecated)]
        Command::BitmapLegacy => {
            warn!("ignoring deprecated command {:?}", command);
        }
        // TODO: how to deduplicate this code in a rusty way?
        Command::BitmapLinear(offset, vec, _) => {
            if !check_bitmap_valid(offset as u16, vec.len()) {
                return true;
            }
            let mut display = display_ref.write().unwrap();
            for bitmap_index in 0..vec.len() {
                let (x, y) = get_coordinates_for_index(offset, bitmap_index);
                display.set(x, y, vec[bitmap_index]);
            }
        }
        Command::BitmapLinearAnd(offset, vec, _) => {
            if !check_bitmap_valid(offset as u16, vec.len()) {
                return true;
            }
            let mut display = display_ref.write().unwrap();
            for bitmap_index in 0..vec.len() {
                let (x, y) = get_coordinates_for_index(offset, bitmap_index);
                let old_value = display.get(x, y);
                display.set(x, y, old_value && vec[bitmap_index]);
            }
        }
        Command::BitmapLinearOr(offset, vec, _) => {
            if !check_bitmap_valid(offset as u16, vec.len()) {
                return true;
            }
            let mut display = display_ref.write().unwrap();
            for bitmap_index in 0..vec.len() {
                let (x, y) = get_coordinates_for_index(offset, bitmap_index);
                let old_value = display.get(x, y);
                display.set(x, y, old_value || vec[bitmap_index]);
            }
        }
        Command::BitmapLinearXor(offset, vec, _) => {
            if !check_bitmap_valid(offset as u16, vec.len()) {
                return true;
            }
            let mut display = display_ref.write().unwrap();
            for bitmap_index in 0..vec.len() {
                let (x, y) = get_coordinates_for_index(offset, bitmap_index);
                let old_value = display.get(x, y);
                display.set(x, y, old_value ^ vec[bitmap_index]);
            }
        }
        Command::CharBrightness(origin, grid) => {
            let mut luma = luma_ref.write().unwrap();
            for inner_y in 0..grid.height() {
                for inner_x in 0..grid.width() {
                    let brightness = grid.get(inner_x, inner_y);
                    luma.set(
                        origin.x + inner_x,
                        origin.y + inner_y,
                        brightness,
                    );
                }
            }
        }
        Command::Brightness(brightness) => {
            luma_ref.write().unwrap().fill(brightness);
        }
        Command::FadeOut => {
            error!("command not implemented: {command:?}")
        }
        Command::Utf8Data(origin, grid) => {
            let mut display = display_ref.write().unwrap();
            print_utf8_data(origin, &grid, utf8_font, &mut display);
        }
    };

    true
}

fn check_bitmap_valid(offset: u16, payload_len: usize) -> bool {
    if offset as usize + payload_len > PIXEL_COUNT {
        error!("bitmap with offset {offset} is too big ({payload_len} bytes)");
        return false;
    }

    true
}

fn print_cp437_data(
    origin: Origin<Tiles>,
    grid: &Cp437Grid,
    font: &Cp437Font,
    display: &mut RwLockWriteGuard<Bitmap>,
) {
    let Origin { x, y, .. } = origin;
    for char_y in 0usize..grid.height() {
        for char_x in 0usize..grid.width() {
            let char_code = grid.get(char_x, char_y);
            trace!(
                "drawing char_code {char_code:#04x} (if this was UTF-8, it would be {})",
                char::from(char_code)
            );

            let tile_x = char_x + x;
            let tile_y = char_y + y;

            let bitmap = font.get_bitmap(char_code);
            if !print_pixel_grid(
                tile_x * TILE_SIZE,
                tile_y * TILE_SIZE,
                bitmap,
                display,
            ) {
                error!("stopping drawing text because char draw failed");
                return;
            }
        }
    }
}

fn print_utf8_data(
    origin: Origin<Tiles>,
    grid: &CharGrid,
    font: &FontRenderer8x8,
    display: &mut RwLockWriteGuard<Bitmap>,
) {
    let Origin { x, y, .. } = origin;
    for char_y in 0usize..grid.height() {
        for char_x in 0usize..grid.width() {
            let char = grid.get(char_x, char_y);
            trace!("drawing {char}");

            let tile_x = char_x + x;
            let tile_y = char_y + y;

            if let Err(e) = font.render(
                char,
                display,
                Origin::new(tile_x * TILE_SIZE, tile_y * TILE_SIZE),
            ) {
                error!("stopping drawing text because char draw failed: {e}");
                return;
            }
        }
    }
}

fn print_pixel_grid(
    offset_x: usize,
    offset_y: usize,
    pixels: &Bitmap,
    display: &mut RwLockWriteGuard<Bitmap>,
) -> bool {
    debug!(
        "printing {}x{} grid at {offset_x} {offset_y}",
        pixels.width(),
        pixels.height()
    );
    for inner_y in 0..pixels.height() {
        for inner_x in 0..pixels.width() {
            let is_set = pixels.get(inner_x, inner_y);
            let x = offset_x + inner_x;
            let y = offset_y + inner_y;

            if x >= display.width() || y >= display.height() {
                error!("stopping pixel grid draw because coordinate {x} {y} is out of bounds");
                return false;
            }

            display.set(x, y, is_set);
        }
    }

    true
}

fn get_coordinates_for_index(offset: usize, index: usize) -> (usize, usize) {
    let pixel_index = offset + index;
    (pixel_index % PIXEL_WIDTH, pixel_index / PIXEL_WIDTH)
}

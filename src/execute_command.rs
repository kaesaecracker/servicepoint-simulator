use std::sync::{RwLock, RwLockWriteGuard};

use log::{debug, error, info, warn};
use servicepoint2::{
    ByteGrid, Command, Origin, PIXEL_COUNT, PIXEL_WIDTH, PixelGrid, TILE_SIZE,
};

use crate::font::BitmapFont;

pub(crate) fn execute_command(
    command: Command,
    font: &BitmapFont,
    display_ref: &RwLock<PixelGrid>,
    luma_ref: &RwLock<ByteGrid>,
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
        Command::BitmapLinearWin(Origin(x, y), pixels) => {
            let mut display = display_ref.write().unwrap();
            print_pixel_grid(x as usize, y as usize, &pixels, &mut display);
        }
        Command::Cp437Data(origin, grid) => {
            let mut display = display_ref.write().unwrap();
            print_cp437_data(origin, &grid, font, &mut display);
        }
        #[allow(deprecated)]
        Command::BitmapLegacy => {
            warn!("ignoring deprecated command {:?}", command);
        }
        // TODO: how to deduplicate this code in a rusty way?
        Command::BitmapLinear(offset, vec, _) => {
            if !check_bitmap_valid(offset, vec.len()) {
                return true;
            }
            let mut display = display_ref.write().unwrap();
            for bitmap_index in 0..vec.len() {
                let (x, y) =
                    get_coordinates_for_index(offset as usize, bitmap_index);
                display.set(x, y, vec.get(bitmap_index));
            }
        }
        Command::BitmapLinearAnd(offset, vec, _) => {
            if !check_bitmap_valid(offset, vec.len()) {
                return true;
            }
            let mut display = display_ref.write().unwrap();
            for bitmap_index in 0..vec.len() {
                let (x, y) =
                    get_coordinates_for_index(offset as usize, bitmap_index);
                let old_value = display.get(x, y);
                display.set(x, y, old_value && vec.get(bitmap_index));
            }
        }
        Command::BitmapLinearOr(offset, vec, _) => {
            if !check_bitmap_valid(offset, vec.len()) {
                return true;
            }
            let mut display = display_ref.write().unwrap();
            for bitmap_index in 0..vec.len() {
                let (x, y) =
                    get_coordinates_for_index(offset as usize, bitmap_index);
                let old_value = display.get(x, y);
                display.set(x, y, old_value || vec.get(bitmap_index));
            }
        }
        Command::BitmapLinearXor(offset, vec, _) => {
            if !check_bitmap_valid(offset, vec.len()) {
                return true;
            }
            let mut display = display_ref.write().unwrap();
            for bitmap_index in 0..vec.len() {
                let (x, y) =
                    get_coordinates_for_index(offset as usize, bitmap_index);
                let old_value = display.get(x, y);
                display.set(x, y, old_value ^ vec.get(bitmap_index));
            }
        }
        Command::CharBrightness(origin, grid) => {
            let Origin(offset_x, offset_y) = origin;
            let offset_x = offset_x as usize;
            let offset_y = offset_y as usize;

            let mut luma = luma_ref.write().unwrap();
            for inner_y in 0..grid.height {
                for inner_x in 0..grid.width {
                    let brightness = grid.get(inner_x, inner_y);
                    luma.set(
                        offset_x + inner_x,
                        offset_y + inner_y,
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
    };

    true
}

fn check_bitmap_valid(offset: u16, payload_len: usize) -> bool {
    if offset as usize + payload_len > PIXEL_COUNT {
        error!(
            "bitmap with offset {offset} is too big ({} bytes)",
            payload_len
        );
        return false;
    }

    true
}

fn print_cp437_data(
    origin: Origin,
    grid: &ByteGrid,
    font: &BitmapFont,
    display: &mut RwLockWriteGuard<PixelGrid>,
) {
    let Origin(x, y) = origin;
    let x = x as usize;
    let y = y as usize;

    for char_y in 0usize..grid.height {
        for char_x in 0usize..grid.width {
            let char_code = grid.get(char_x, char_y);

            let tile_x = char_x + x;
            let tile_y = char_y + y;

            let bitmap = font.get_bitmap(char_code);
            print_pixel_grid(
                tile_x * TILE_SIZE as usize,
                tile_y * TILE_SIZE as usize,
                bitmap,
                display,
            );
        }
    }
}

fn print_pixel_grid(
    offset_x: usize,
    offset_y: usize,
    pixels: &PixelGrid,
    display: &mut RwLockWriteGuard<PixelGrid>,
) {
    debug!(
        "printing {}x{} grid at {offset_x} {offset_y}",
        pixels.width, pixels.height
    );
    for inner_y in 0..pixels.height {
        for inner_x in 0..pixels.width {
            let is_set = pixels.get(inner_x, inner_y);
            display.set(offset_x + inner_x, offset_y + inner_y, is_set);
        }
    }
}

fn get_coordinates_for_index(offset: usize, index: usize) -> (usize, usize) {
    let pixel_index = offset + index;
    (
        pixel_index % PIXEL_WIDTH as usize,
        pixel_index / PIXEL_WIDTH as usize,
    )
}

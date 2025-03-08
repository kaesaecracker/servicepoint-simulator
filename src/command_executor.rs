use crate::{
    command_executor::ExecutionResult::{Failure, Shutdown, Success},
    cp437_font::Cp437Font,
    font_renderer::FontRenderer8x8,
};
use log::{debug, error, info, trace, warn};
use servicepoint::{
    BinaryOperation, BitVec, BitVecCommand, Bitmap, BitmapCommand,
    BrightnessCommand, BrightnessGrid, BrightnessGridCommand, CharGrid,
    CharGridCommand, Cp437Grid, Cp437GridCommand, Grid, Offset, Origin, Tiles,
    TypedCommand, PIXEL_COUNT, PIXEL_WIDTH, TILE_SIZE,
};
use std::{
    ops::{BitAnd, BitOr, BitXor},
    sync::RwLock,
};

#[derive(Debug)]
pub struct CommandExecutor<'t> {
    display: &'t RwLock<Bitmap>,
    luma: &'t RwLock<BrightnessGrid>,
    cp437_font: Cp437Font,
    font_renderer: FontRenderer8x8,
}

#[must_use]
pub enum ExecutionResult {
    Success,
    Failure,
    Shutdown,
}

impl<'t> CommandExecutor<'t> {
    pub fn new(
        display: &'t RwLock<Bitmap>,
        luma: &'t RwLock<BrightnessGrid>,
        font_renderer: FontRenderer8x8,
    ) -> Self {
        CommandExecutor {
            display,
            luma,
            font_renderer,
            cp437_font: Cp437Font::default(),
        }
    }

    pub(crate) fn execute(&self, command: TypedCommand) -> ExecutionResult {
        debug!("received {command:?}");
        match command {
            TypedCommand::Clear(_) => {
                info!("clearing display");
                self.display.write().unwrap().fill(false);
                Success
            }
            TypedCommand::HardReset(_) => {
                warn!("display shutting down");
                Shutdown
            }
            TypedCommand::Bitmap(BitmapCommand {
                                     origin: Origin { x, y, .. },
                                     bitmap,
                                     ..
                                 }) => self.print_pixel_grid(x, y, &bitmap),
            TypedCommand::Cp437Grid(Cp437GridCommand { origin, grid }) => {
                self.print_cp437_data(origin, &grid)
            }
            #[allow(deprecated)]
            TypedCommand::BitmapLegacy(_) => {
                warn!("ignoring deprecated command {:?}", command);
                Failure
            }
            TypedCommand::BitVec(command) => {
                let BitVecCommand {
                    offset,
                    bitvec,
                    operation,
                    ..
                } = command;
                fn overwrite(_: bool, new: bool) -> bool {
                    new
                }
                let operation = match operation {
                    BinaryOperation::Overwrite => overwrite,
                    BinaryOperation::And => BitAnd::bitand,
                    BinaryOperation::Or => BitOr::bitor,
                    BinaryOperation::Xor => BitXor::bitxor,
                };
                self.execute_bitmap_linear(offset, bitvec, operation)
            }
            TypedCommand::BrightnessGrid(BrightnessGridCommand {
                                             origin,
                                             grid,
                                         }) => self.execute_char_brightness(origin, grid),
            TypedCommand::Brightness(BrightnessCommand { brightness }) => {
                self.luma.write().unwrap().fill(brightness);
                Success
            }
            TypedCommand::FadeOut(_) => {
                error!("command not implemented: {command:?}");
                Success
            }
            TypedCommand::CharGrid(CharGridCommand { origin, grid }) => {
                self.print_utf8_data(origin, &grid)
            }
        }
    }

    fn execute_char_brightness(
        &self,
        origin: Origin<Tiles>,
        grid: BrightnessGrid,
    ) -> ExecutionResult {
        let mut luma = self.luma.write().unwrap();
        for inner_y in 0..grid.height() {
            for inner_x in 0..grid.width() {
                let brightness = grid.get(inner_x, inner_y);
                luma.set(origin.x + inner_x, origin.y + inner_y, brightness);
            }
        }
        Success
    }

    fn execute_bitmap_linear<Op>(
        &self,
        offset: Offset,
        vec: BitVec,
        op: Op,
    ) -> ExecutionResult
    where
        Op: Fn(bool, bool) -> bool,
    {
        if !Self::check_bitmap_valid(offset as u16, vec.len()) {
            return Failure;
        }
        let mut display = self.display.write().unwrap();
        for bitmap_index in 0..vec.len() {
            let (x, y) = Self::get_coordinates_for_index(offset, bitmap_index);
            let old_value = display.get(x, y);
            display.set(x, y, op(old_value, vec[bitmap_index]));
        }
        Success
    }

    fn check_bitmap_valid(offset: u16, payload_len: usize) -> bool {
        if offset as usize + payload_len > PIXEL_COUNT {
            error!(
                "bitmap with offset {offset} is too big ({payload_len} bytes)"
            );
            return false;
        }

        true
    }

    fn print_cp437_data(
        &self,
        origin: Origin<Tiles>,
        grid: &Cp437Grid,
    ) -> ExecutionResult {
        let font = &self.cp437_font;
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

                match self.print_pixel_grid(
                    tile_x * TILE_SIZE,
                    tile_y * TILE_SIZE,
                    &font[char_code],
                ) {
                    Success => {}
                    Failure => {
                        error!(
                            "stopping drawing text because char draw failed"
                        );
                        return Failure;
                    }
                    Shutdown => return Shutdown,
                }
            }
        }

        Success
    }

    fn print_utf8_data(
        &self,
        origin: Origin<Tiles>,
        grid: &CharGrid,
    ) -> ExecutionResult {
        let mut display = self.display.write().unwrap();

        let Origin { x, y, .. } = origin;
        for char_y in 0usize..grid.height() {
            for char_x in 0usize..grid.width() {
                let char = grid.get(char_x, char_y);
                trace!("drawing {char}");

                let tile_x = char_x + x;
                let tile_y = char_y + y;

                if let Err(e) = self.font_renderer.render(
                    char,
                    &mut display,
                    Origin::new(tile_x * TILE_SIZE, tile_y * TILE_SIZE),
                ) {
                    error!(
                        "stopping drawing text because char draw failed: {e}"
                    );
                    return Failure;
                }
            }
        }

        Success
    }

    fn print_pixel_grid(
        &self,
        offset_x: usize,
        offset_y: usize,
        pixels: &Bitmap,
    ) -> ExecutionResult {
        debug!(
            "printing {}x{} grid at {offset_x} {offset_y}",
            pixels.width(),
            pixels.height()
        );
        let mut display = self.display.write().unwrap();
        for inner_y in 0..pixels.height() {
            for inner_x in 0..pixels.width() {
                let is_set = pixels.get(inner_x, inner_y);
                let x = offset_x + inner_x;
                let y = offset_y + inner_y;

                if x >= display.width() || y >= display.height() {
                    error!("stopping pixel grid draw because coordinate {x} {y} is out of bounds");
                    return Failure;
                }

                display.set(x, y, is_set);
            }
        }

        Success
    }

    fn get_coordinates_for_index(
        offset: usize,
        index: usize,
    ) -> (usize, usize) {
        let pixel_index = offset + index;
        (pixel_index % PIXEL_WIDTH, pixel_index / PIXEL_WIDTH)
    }
}

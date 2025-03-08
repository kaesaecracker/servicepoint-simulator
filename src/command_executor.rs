use crate::{
    command_executor::ExecutionResult::{Failure, Shutdown, Success},
    cp437_font::Cp437Font,
    font_renderer::FontRenderer8x8,
};
use log::{debug, error, info, trace, warn};
use servicepoint::{
    BinaryOperation, BitVecCommand, Bitmap, BitmapCommand, BrightnessCommand,
    BrightnessGrid, BrightnessGridCommand, CharGridCommand, ClearCommand,
    CompressionCode, Cp437GridCommand, FadeOutCommand, Grid, HardResetCommand,
    Origin, TypedCommand, PIXEL_COUNT, PIXEL_WIDTH, TILE_SIZE,
};
use std::{
    ops::{BitAnd, BitOr, BitXor},
    sync::RwLock,
};

#[derive(Debug)]
pub struct CommandExecutionContext<'t> {
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

pub trait CommandExecute {
    fn execute(&self, context: &CommandExecutionContext) -> ExecutionResult;
}

impl CommandExecute for ClearCommand {
    fn execute(&self, context: &CommandExecutionContext) -> ExecutionResult {
        info!("clearing display");
        context.display.write().unwrap().fill(false);
        Success
    }
}

impl CommandExecute for BitmapCommand {
    fn execute(&self, context: &CommandExecutionContext) -> ExecutionResult {
        let Self {
            origin:
            Origin {
                x: offset_x,
                y: offset_y,
                ..
            },
            bitmap: pixels,
            ..
        } = self;
        debug!(
            "printing {}x{} grid at {offset_x} {offset_y}",
            pixels.width(),
            pixels.height()
        );
        let mut display = context.display.write().unwrap();
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
}

impl CommandExecute for HardResetCommand {
    fn execute(&self, _: &CommandExecutionContext) -> ExecutionResult {
        warn!("display shutting down");
        Shutdown
    }
}

impl CommandExecute for BitVecCommand {
    fn execute(&self, context: &CommandExecutionContext) -> ExecutionResult {
        let BitVecCommand {
            offset,
            bitvec,
            operation,
            ..
        } = self;
        fn overwrite(_: bool, new: bool) -> bool {
            new
        }
        let operation = match operation {
            BinaryOperation::Overwrite => overwrite,
            BinaryOperation::And => BitAnd::bitand,
            BinaryOperation::Or => BitOr::bitor,
            BinaryOperation::Xor => BitXor::bitxor,
        };

        if self.offset + bitvec.len() > PIXEL_COUNT {
            error!(
                "bitmap with offset {offset} is too big ({} bytes)",
                bitvec.len()
            );
            return Failure;
        }

        let mut display = context.display.write().unwrap();
        for bitmap_index in 0..bitvec.len() {
            let pixel_index = offset + bitmap_index;
            let (x, y) = (pixel_index % PIXEL_WIDTH, pixel_index / PIXEL_WIDTH);
            let old_value = display.get(x, y);
            display.set(x, y, operation(old_value, bitvec[bitmap_index]));
        }
        Success
    }
}

impl CommandExecute for Cp437GridCommand {
    fn execute(&self, context: &CommandExecutionContext) -> ExecutionResult {
        let Cp437GridCommand { origin, grid } = self;
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

                let execute_result = BitmapCommand {
                    origin: Origin::new(tile_x * TILE_SIZE, tile_y * TILE_SIZE),
                    bitmap: context.cp437_font[char_code].clone(),
                    compression: CompressionCode::default(),
                }
                    .execute(context);
                match execute_result {
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
}

#[allow(deprecated)]
impl CommandExecute for servicepoint::BitmapLegacyCommand {
    fn execute(&self, _: &CommandExecutionContext) -> ExecutionResult {
        warn!("ignoring deprecated command {:?}", self);
        Failure
    }
}

impl CommandExecute for BrightnessGridCommand {
    fn execute(&self, context: &CommandExecutionContext) -> ExecutionResult {
        let BrightnessGridCommand { origin, grid } = self;
        let mut luma = context.luma.write().unwrap();
        for inner_y in 0..grid.height() {
            for inner_x in 0..grid.width() {
                let brightness = grid.get(inner_x, inner_y);
                luma.set(origin.x + inner_x, origin.y + inner_y, brightness);
            }
        }
        Success
    }
}

impl CommandExecute for CharGridCommand {
    fn execute(&self, context: &CommandExecutionContext) -> ExecutionResult {
        let CharGridCommand { origin, grid } = self;
        let mut display = context.display.write().unwrap();

        let Origin { x, y, .. } = origin;
        for char_y in 0usize..grid.height() {
            for char_x in 0usize..grid.width() {
                let char = grid.get(char_x, char_y);
                trace!("drawing {char}");

                let tile_x = char_x + x;
                let tile_y = char_y + y;

                if let Err(e) = context.font_renderer.render(
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
}

impl CommandExecute for BrightnessCommand {
    fn execute(&self, context: &CommandExecutionContext) -> ExecutionResult {
        context.luma.write().unwrap().fill(self.brightness);
        Success
    }
}

impl CommandExecute for FadeOutCommand {
    fn execute(&self, _: &CommandExecutionContext) -> ExecutionResult {
        error!("command not implemented: {self:?}");
        Success
    }
}

impl CommandExecute for TypedCommand {
    fn execute(&self, context: &CommandExecutionContext) -> ExecutionResult {
        match self {
            TypedCommand::Clear(command) => command.execute(context),
            TypedCommand::HardReset(command) => command.execute(context),
            TypedCommand::Bitmap(command) => command.execute(context),
            TypedCommand::Cp437Grid(command) => command.execute(context),
            #[allow(deprecated)]
            TypedCommand::BitmapLegacy(command) => command.execute(context),
            TypedCommand::BitVec(command) => command.execute(context),
            TypedCommand::BrightnessGrid(command) => command.execute(context),
            TypedCommand::Brightness(command) => command.execute(context),
            TypedCommand::FadeOut(command) => command.execute(context),
            TypedCommand::CharGrid(command) => command.execute(context),
        }
    }
}

impl<'t> CommandExecutionContext<'t> {
    pub fn new(
        display: &'t RwLock<Bitmap>,
        luma: &'t RwLock<BrightnessGrid>,
        font_renderer: FontRenderer8x8,
    ) -> Self {
        CommandExecutionContext {
            display,
            luma,
            font_renderer,
            cp437_font: Cp437Font::default(),
        }
    }
}

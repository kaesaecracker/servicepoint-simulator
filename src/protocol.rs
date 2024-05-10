use std::mem::size_of;

use num_derive::{FromPrimitive, ToPrimitive};
#[repr(u16)]
#[derive(Debug, FromPrimitive, ToPrimitive, Default)]
pub enum DisplayCommandCode {
    #[default]
    Clear = 0x0002,
    Cp437data = 0x0003,
    CharBrightness = 0x0005,
    Brightness = 0x0007,
    HardReset = 0x000b,
    FadeOut = 0x000d,
    BitmapLegacy = 0x0010,
    BitmapLinear = 0x0012,
    BitmapLinearWin = 0x0013,
    BitmapLinearAnd = 0x0014,
    BitmapLinearOr = 0x0015,
    BitmapLinearXor = 0x0016,
}

impl DisplayCommandCode {
    pub fn from_primitive(value: u16) -> Option<Self> {
        num::FromPrimitive::from_u16(value)
    }

    pub fn to_primitive(&self) -> u16 {
        num::ToPrimitive::to_u16(self).unwrap()
    }
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct HdrWindow {
    pub command: DisplayCommandCode,
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
}

/*
#[repr(C)]
pub struct HdrBitmap {
    pub command: DisplayCommand,
    pub offset: u16,
    pub length: u16,
    pub subcommand: DisplaySubcommand,
    reserved: u16,
}
*/

#[repr(u16)]
#[derive(Debug, FromPrimitive, ToPrimitive)]
pub enum DisplaySubcommand {
    SubCmdBitmapNormal = 0x0,
    SubCmdBitmapCompressZ = 0x677a,
    SubCmdBitmapCompressBz = 0x627a,
    SubCmdBitmapCompressLz = 0x6c7a,
    SubCmdBitmapCompressZs = 0x7a73,
}

#[derive(Debug)]
pub enum ReadHeaderError {
    BufferTooSmall,
    WrongCommandEndianness(u16, DisplayCommandCode),
    InvalidCommand(u16),
}

pub fn read_header(buffer: &[u8]) -> Result<HdrWindow, ReadHeaderError> {
    assert_eq!(size_of::<HdrWindow>(), 10, "invalid struct size");

    if buffer.len() < size_of::<HdrWindow>() {
        return Err(ReadHeaderError::BufferTooSmall);
    }

    let command_u16 = read_beu16(&buffer[0..=1]);
    return match DisplayCommandCode::from_primitive(command_u16) {
        Some(command) => Ok(HdrWindow {
            command,
            x: read_beu16(&buffer[2..=3]),
            y: read_beu16(&buffer[4..=5]),
            w: read_beu16(&buffer[6..=7]),
            h: read_beu16(&buffer[8..=9]),
        }),
        None => {
            let maybe_command = DisplayCommandCode::from_primitive(u16::swap_bytes(command_u16));
            return match maybe_command {
                None => Err(ReadHeaderError::InvalidCommand(command_u16)),
                Some(command) => Err(ReadHeaderError::WrongCommandEndianness(
                    command_u16,
                    command,
                )),
            };
        }
    };
}

fn read_beu16(buffer: &[u8]) -> u16 {
    let buffer: [u8; 2] = buffer
        .try_into()
        .expect("cannot read u16 from buffer with size != 2");
    return u16::from_be_bytes(buffer);
}

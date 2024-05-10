use num_derive::{FromPrimitive, ToPrimitive};
use servicepoint2::DisplayCommandCode;
use std::mem::size_of;

#[repr(C)]
#[derive(Debug)]
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

impl HdrWindow {
    pub fn from_buffer(buffer: &[u8]) -> Result<HdrWindow, ReadHeaderError> {
        assert_eq!(size_of::<HdrWindow>(), 10, "invalid struct size");

        if buffer.len() < size_of::<HdrWindow>() {
            return Err(ReadHeaderError::BufferTooSmall);
        }

        let command_u16 = Self::read_beu16(&buffer[0..=1]);
        return match DisplayCommandCode::from_primitive(command_u16) {
            Some(command) => Ok(HdrWindow {
                command,
                x: Self::read_beu16(&buffer[2..=3]),
                y: Self::read_beu16(&buffer[4..=5]),
                w: Self::read_beu16(&buffer[6..=7]),
                h: Self::read_beu16(&buffer[8..=9]),
            }),
            None => {
                let maybe_command =
                    DisplayCommandCode::from_primitive(u16::swap_bytes(command_u16));
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
}

use servicepoint::{Bitmap, DataRef, TILE_SIZE};
use std::ops::Index;

const CHAR_COUNT: usize = u8::MAX as usize + 1;

#[derive(Debug)]
pub struct Cp437Font {
    bitmaps: [Bitmap; CHAR_COUNT],
}

impl Cp437Font {
    pub fn new(bitmaps: [Bitmap; CHAR_COUNT]) -> Self {
        Self { bitmaps }
    }
}

impl Default for Cp437Font {
    fn default() -> Self {
        let mut bitmaps =
            core::array::from_fn(|_| Bitmap::new(TILE_SIZE, TILE_SIZE));

        for (char_code, bitmap) in bitmaps.iter_mut().enumerate() {
            let bits = CP437_FONT_LINEAR[char_code];
            let mut bytes = bits.to_be_bytes();
            bytes.reverse();
            bitmap.data_ref_mut().copy_from_slice(bytes.as_slice());
        }

        Self::new(bitmaps)
    }
}

impl Index<u8> for Cp437Font {
    type Output = Bitmap;

    fn index(&self, char_code: u8) -> &Self::Output {
        &self.bitmaps[char_code as usize]
    }
}

/// Font from the display firmware `cape-cccb-apd/cp437font_linear.h`
pub(crate) const CP437_FONT_LINEAR: [u64; 256] = [
    0x0000000000000000, // 0x00
    0x003854ba82aa4438, // 0x01
    0x003844bafed67c38, // 0x02
    0x0010387cfeee4400, // 0x03
    0x0010387cfe7c3810, // 0x04
    0x003810d6fefe3838, // 0x05
    0x003810d6fe7c3810, // 0x06
    0x0000387c7c7c3800, // 0x07
    0x00fec6828282c6fe, // 0x08
    0x0000384444443800, // 0x09
    0x00fec6bababac6fe, // 0x0a
    0x007088888a7a061e, // 0x0b
    0x1038103844444438, // 0x0c
    0x0030301010141418, // 0x0d
    0xc0c64642724e720e, // 0x0e
    0x00927c44c6447c92, // 0x0f
    0x00c0f0fcfefcf0c0, // 0x10
    0x00061e7efe7e1e06, // 0x11
    0x1038541010543810, // 0x12
    0x2800282828282828, // 0x13
    0x000e0a0a7a8a8a7e, // 0x14
    0x0038441c28704438, // 0x15
    0x000000ffff000000, // 0x16
    0xfe10385410543810, // 0x17
    0x1010101010543810, // 0x18
    0x1038541010101010, // 0x19
    0x00000804fe040800, // 0x1a
    0x00002040fe402000, // 0x1b
    0xffffc0c0c0c0c0c0, // 0x1c
    0x00002844fe442800, // 0x1d
    0x00fefe7c7c383810, // 0x1e
    0x001038387c7cfefe, // 0x1f
    0x0000000000000000, // 0x20
    0x1000101010101010, // 0x21
    0x0000000000505028, // 0x22
    0x00247e24247e2400, // 0x23
    0x1038541830543810, // 0x24
    0x00844a2a54a8a442, // 0x25
    0x003a444a32484830, // 0x26
    0x0000000000201010, // 0x27
    0x0810202020201008, // 0x28
    0x2010080808081020, // 0x29
    0x0010543854100000, // 0x2a
    0x0010107c10100000, // 0x2b
    0x2010100000000000, // 0x2c
    0x0000007c00000000, // 0x2d
    0x0010000000000000, // 0x2e
    0x4020201010080804, // 0x2f
    0x0038444454444438, // 0x30
    0x007c101010503010, // 0x31
    0x007c201008044438, // 0x32
    0x003844043810087c, // 0x33
    0x0004047e24140c04, // 0x34
    0x003844047840407c, // 0x35
    0x003844447840201c, // 0x36
    0x004020100804047c, // 0x37
    0x0038444438444438, // 0x38
    0x007008043c444438, // 0x39
    0x0000100000100000, // 0x3a
    0x2010100000100000, // 0x3b
    0x0006186018060000, // 0x3c
    0x00007c007c000000, // 0x3d
    0x00c0300c30c00000, // 0x3e
    0x1000384038044438, // 0x3f
    0x001c2248564a221c, // 0x40
    0x0042427e24241818, // 0x41
    0x007c42427c444478, // 0x42
    0x001c22404040221c, // 0x43
    0x0078444242424478, // 0x44
    0x007e40407e40407e, // 0x45
    0x004040407e40407e, // 0x46
    0x001c22424e40221c, // 0x47
    0x004242427e424242, // 0x48
    0x007c10101010107c, // 0x49
    0x003844040404047c, // 0x4a
    0x0042444870484442, // 0x4b
    0x007e404040404040, // 0x4c
    0x0082828292aac682, // 0x4d
    0x004242464a526242, // 0x4e
    0x0018244242422418, // 0x4f
    0x004040407c42427c, // 0x50
    0x001a244a42422418, // 0x51
    0x004244487c42427c, // 0x52
    0x003c42023c40423c, // 0x53
    0x00101010101010fe, // 0x54
    0x003c424242424242, // 0x55
    0x0010282844448282, // 0x56
    0x0044446caa929282, // 0x57
    0x0082442810284482, // 0x58
    0x0010101010284482, // 0x59
    0x007e20100804027e, // 0x5a
    0x3820202020202038, // 0x5b
    0x0408081010202040, // 0x5c
    0x3808080808080838, // 0x5d
    0x0000000000442810, // 0x5e
    0x007e000000000000, // 0x5f
    0x0000000000080810, // 0x60
    0x003c443c04380000, // 0x61
    0x0038444444784040, // 0x62
    0x0038444044380000, // 0x63
    0x003c4444443c0404, // 0x64
    0x003c407844380000, // 0x65
    0x2020202078202418, // 0x66
    0x78043c44443e0000, // 0x67
    0x0044444464584040, // 0x68
    0x001c101010700010, // 0x69
    0x38440404047c0010, // 0x6a
    0x0022243828242020, // 0x6b
    0x0018242020202020, // 0x6c
    0x0054545454780000, // 0x6d
    0x0044444464580000, // 0x6e
    0x0038444444380000, // 0x6f
    0x4040784444780000, // 0x70
    0x04043c44443c0000, // 0x71
    0x0040404064580000, // 0x72
    0x00384418201c0000, // 0x73
    0x0018242020782020, // 0x74
    0x0038444444440000, // 0x75
    0x0010282844440000, // 0x76
    0x0028285454440000, // 0x77
    0x0044281028440000, // 0x78
    0x38043c4444440000, // 0x79
    0x007c2010087c0000, // 0x7a
    0x0c0808083008080c, // 0x7b
    0x1010101010101010, // 0x7c
    0x301010100c101030, // 0x7d
    0x0000004c32000000, // 0x7e
    0xfe82828282442810, // 0x7f
    0x18083c428080423c, // 0x80
    0x0038444444440028, // 0x81
    0x003c407844381008, // 0x82
    0x003c443c04382810, // 0x83
    0x003c443c04380044, // 0x84
    0x003c443c04381020, // 0x85
    0x003c443c04382838, // 0x86
    0x1038444044380000, // 0x87
    0x003c407844382810, // 0x88
    0x003c407844380044, // 0x89
    0x003c407844381020, // 0x8a
    0x001c101010700028, // 0x8b
    0x001c101010702810, // 0x8c
    0x001c101010701020, // 0x8d
    0x0082827c44282892, // 0x8e
    0x0082827c44281038, // 0x8f
    0x00fe80fe80fe1008, // 0x90
    0x007e907c126c0000, // 0x91
    0x008e88784e28281e, // 0x92
    0x0038444438002810, // 0x93
    0x0038444444380028, // 0x94
    0x0038444438001020, // 0x95
    0x0038444444002810, // 0x96
    0x0038444444001020, // 0x97
    0x38043c4444440028, // 0x98
    0x0038448282443882, // 0x99
    0x0038448282820082, // 0x9a
    0x1038444044381010, // 0x9b
    0x00fc4240f0404438, // 0x9c
    0x107c107c10284482, // 0x9d
    0x001c22f840f8221c, // 0x9e
    0x205010103810120c, // 0x9f
    0x003c443c04381008, // 0xa0
    0x001c101010701008, // 0xa1
    0x0038444438001008, // 0xa2
    0x0038444444001008, // 0xa3
    0x0044446458004834, // 0xa4
    0x00464a5262424834, // 0xa5
    0x0000007090701060, // 0xa6
    0x0000007088888870, // 0xa7
    0x3844403804380010, // 0xa8
    0x0040407c00000000, // 0xa9
    0x0004047c00000000, // 0xaa
    0x0e84422c5048c442, // 0xab
    0x049e542c5448c442, // 0xac
    0x1010101010100010, // 0xad
    0x0024489048240000, // 0xae
    0x0048241224480000, // 0xaf
    0x1122448811224488, // 0xb0
    0x55aa55aa55aa55aa, // 0xb1
    0xddbb77eeddbb77ee, // 0xb2
    0x1010101010101010, // 0xb3
    0x101020c020101010, // 0xb4
    0x1020c000c0201010, // 0xb5
    0x2828488848282828, // 0xb6
    0x282850e000000000, // 0xb7
    0x1030d020c0000000, // 0xb8
    0x2848880888482828, // 0xb9
    0x2828282828282828, // 0xba
    0x2828c810e0000000, // 0xbb
    0x000000e010c82828, // 0xbc
    0x000000e050282828, // 0xbd
    0x0000c020d0301010, // 0xbe
    0x101020c000000000, // 0xbf
    0x0000000708101010, // 0xc0
    0x000000c728101010, // 0xc1
    0x101028c700000000, // 0xc2
    0x1010080708101010, // 0xc3
    0x000000ff00000000, // 0xc4
    0x101028c728101010, // 0xc5
    0x1008070007081010, // 0xc6
    0x2828242324282828, // 0xc7
    0x00000f1027282828, // 0xc8
    0x282827100f000000, // 0xc9
    0x0000ff0083442828, // 0xca
    0x28448300ff000000, // 0xcb
    0x2824232023242828, // 0xcc
    0x0000ff00ff000000, // 0xcd
    0x2844932893442828, // 0xce
    0x0000ff00c7281010, // 0xcf
    0x0000008344282828, // 0xd0
    0x1028c700ff000000, // 0xd1
    0x2828448300000000, // 0xd2
    0x0000000f14282828, // 0xd3
    0x0000070817181010, // 0xd4
    0x1018170807000000, // 0xd5
    0x2828140f00000000, // 0xd6
    0x2828448344282828, // 0xd7
    0x1028c700c7281010, // 0xd8
    0x000000c020101010, // 0xd9
    0x1010080700000000, // 0xda
    0xffffffffffffffff, // 0xdb
    0xffffffff00000000, // 0xdc
    0xf0f0f0f0f0f0f0f0, // 0xdd
    0x0f0f0f0f0f0f0f0f, // 0xde
    0x00000000ffffffff, // 0xdf
    0x0076888888740200, // 0xe0
    0x5844444458484830, // 0xe1
    0x00e04040404242fe, // 0xe2
    0x00242828a87c0000, // 0xe3
    0x00fe8240204082fe, // 0xe4
    0x00384444443e0000, // 0xe5
    0x405a644444440000, // 0xe6
    0x0010282020fc0000, // 0xe7
    0xfe103854543810fe, // 0xe8
    0x003844aabaaa4438, // 0xe9
    0x00ee448282824438, // 0xea
    0x007088887012221c, // 0xeb
    0x00006c92926c0000, // 0xec
    0x10107c92924c0000, // 0xed
    0x0038403040380000, // 0xee
    0x0082828282824438, // 0xef
    0x00fe00fe00fe0000, // 0xf0
    0x007c10107c101000, // 0xf1
    0x007e006018061860, // 0xf2
    0x007e000618601806, // 0xf3
    0x101010101010120c, // 0xf4
    0x6090101010101010, // 0xf5
    0x0010007c00100000, // 0xf6
    0x000c926c92600000, // 0xf7
    0x0000000030484830, // 0xf8
    0x0000103810000000, // 0xf9
    0x0000001000000000, // 0xfa
    0x10102828a4440202, // 0xfb
    0x00000000484848b0, // 0xfc
    0x0000000070201060, // 0xfd
    0x00007c7c7c7c7c00, // 0xfe
    0x0000000000000000, // 0xff
];

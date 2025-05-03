# servicepoint-simulator

[![Releases](https://git.berlin.ccc.de/servicepoint/servicepoint-simulator/badges/release.svg)](https://git.berlin.ccc.de/servicepoint/servicepoint-simulator/releases)
[![crates.io](https://img.shields.io/crates/v/servicepoint-simulator.svg)](https://crates.io/crates/servicepoint-simulator)
[![Crates.io Total Downloads](https://img.shields.io/crates/d/servicepoint-simulator)](https://crates.io/crates/servicepoint-simulator)
[![GPLv3 licensed](https://img.shields.io/crates/l/servicepoint-simulator)](./LICENSE)
[![CI](https://git.berlin.ccc.de/servicepoint/servicepoint-simulator/badges/workflows/rust.yml/badge.svg)](https://git.berlin.ccc.de/servicepoint/servicepoint-simulator)

A simulator for the CCCB service point display.

![example render](example_render.png)

In CCCB, there is a big LED matrix screen you can send images to via UDP.
This crate contains an application that can receive packages in the same binary format and display the contents to the
user.

Use cases:

- getting error messages for invalid packages (instead of nothing happening on the display)
- test your project when outside CCCB
- test your project while other people are using the display

Uses the [servicepoint](https://github.com/cccb/servicepoint) library for reading the packets.
The screenshot above shows the output of two example projects running in parallel (game_of_life and random_brightness).

This repository moved
to [git.berlin.ccc.de/servicepoint/servicepoint-simulator](https://git.berlin.ccc.de/servicepoint/servicepoint-simulator/).
The [GitHub repository](https://github.com/kaesaecracker/servicepoint-simulator) will remain as a mirror.

## Running

With cargo installed: `cargo install servicepoint-simulator`

With nix flakes: `nix run github:kaesaecracker/servicepoint-simulator`

You can also check out this repository and use `cargo run --release`.
Make sure to run a release build, because a debug build _way_ slower.

## Command line arguments

```
Usage: servicepoint-simulator [OPTIONS]

Options:
      --bind <BIND>  address and port to bind to [default: 0.0.0.0:2342]
  -f, --font <FONT>  The name of the font family to use. This defaults to the system monospace font.
  -s, --spacers      add spacers between tile rows to simulate gaps in real display
  -r, --red          Use the red color channel
  -g, --green        Use the green color channel
  -b, --blue         Use the blue color channel
  -v, --verbose      Set default log level lower. You can also change this via the RUST_LOG environment variable.
  -h, --help         Print help
```

See [env_logger](https://docs.rs/env_logger/latest/env_logger/) to configure logging.

Because this program renders to an RGB pixel buffer, you can enjoy the following additional features not available on
the real display:

- enable or disable the empty space between tile rows (`./servicepoint-simulator --spacers` to enable)
- render pixels in red, green, blue or a combination of the three (`./servicepoint-simulator -rgb` for white pixels)

## Known differences

- The font used for displaying UTF-8 text is your default system monospace font, rendered to 8x8 pixels
- The brightness levels will look linear in the simulator
- Some commands will be executed in part on the real display and then produce an error (in a console you cannot see)
  while the simulator refuses to execute the whole command

## Contributing

Contributions are accepted in any form (issues, documentation, feature requests, code, reviews, ...).

All creatures welcome.

## Legal stuff

The included font is https://int10h.org/oldschool-pc-fonts/fontlist/font?ibm_bios (included in the download
from https://int10h.org/oldschool-pc-fonts/download/). The font is CC BY-SA 4.0.

For everything else see the LICENSE file.

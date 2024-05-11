# pixel-receiver-rs

An emulator for the CCCB airport display. 

![example render](example_render.png)

In CCCB, there is a big LED matrix screen you can send images to via UDP. This project aims to build a working an application that can receive packages in the same binary format and display the contents to the user.

Use cases:
- getting error messages for invalid packages
- test your project when outside of CCCB
- test your project while other people are using the display

Uses the [servicepoint](https://github.com/cccb/servicepoint) library for reading the packets. Currently only works with my [fork](https://github.com/kaesaecracker/servicepoint).
The screenshot above shows the output of two example projects running in parallel (game_of_life and random_brightness).

## Legal stuff

The included font is https://int10h.org/oldschool-pc-fonts/fontlist/font?ibm_bios (included in the download from https://int10h.org/oldschool-pc-fonts/download/). The font is CC BY-SA 4.0.

For everything else see the LICENSE file.

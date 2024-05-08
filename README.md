# pixel-receiver-rs

An emulator for the CCCB airport display. 

In CCCB, there is a big LED matrix screen you can send images to via UDP. This project aims to build a working an application that can receive packages in the same binary format and display the contents to the user.

Use cases:
- getting error messages for invalid packages
- test your project when outside of CCCB
- test your project while other people are using the display

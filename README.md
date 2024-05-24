# SEGA Genesis RGBlast Processor - Rust client

A Rust Client for playing SEGA Genesis games on PC via RGB stream.

## Description

The SEGA Genesis RGBlast Processor is a Raspi Pico based video capture device that directly digitizes the RGB output of a SEGA Genesis and delivers it via USB to a host computer. Please see the hardware project for more information (TBD)

This is the software client for that hardware, which receives and converts the stream into video.

At the moment this means 6 bits per pixel, ~30FPS.

https://user-images.githubusercontent.com/127321359/224422179-d3227273-6468-4ef0-991d-ddc945e091db.mp4

## Getting Started

FTDI D2XX Drivers need to be installed from their official website: https://ftdichip.com/drivers/d2xx-drivers/

Obviously you'll also need Rust installed (https://www.rust-lang.org/).


### Dependencies

* Rust (1.6+?)
* FTDI D2XXX Drivers
* Windows (theoretically this works on Mac/PC too)

## Help

On certain platforms you cannot have both the D2XX and VCP (Virtual Com Port) drivers installed. Try removing the VCP drivers if you have issues.

## License

This project is licensed under the MIT License - see the LICENSE file for details

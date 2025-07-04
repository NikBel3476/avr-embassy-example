# avr-embassy-example

Rust examle project for the _Arduino Uno_ with [embassy](https://github.com/embassy-rs/embassy).

## Build Instructions
1. Install prerequisites as described in the [`avr-hal` README] (`avr-gcc`, `avr-libc`, `avrdude`, [`ravedude`]).
2. Run `cargo build` to build the firmware.
3. Run `RAVEDUDE_PORT=<port> cargo run --release` to build and flash the firmware to a connected board.
4. `ravedude` will open a console session after flashing where you can interact
with the UART console of your board.

[`avr-hal` README]: https://github.com/Rahix/avr-hal#readme
[`ravedude`]: https://crates.io/crates/ravedude

## License
Licensed under either of
- Apache License, Version 2.0
([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license
([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

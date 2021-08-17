# io_tee

[![Crates.io](https://img.shields.io/crates/v/io_tee)](https://crates.io/crates/io_tee)

A very simple library which supports teeing `Read`, `BufRead`, and `Seek` readers and `Write`rs.
Implementations respect the underlying reader or writer's overriden methods.

For convenience, `ReadExt` and `WriteExt` traits are provided to easily construct `TeeReader` and `TeeWriter` from
existing `Read` and `Write` streams.

## License
Licensed under either of
- Apache License, Version 2.0
- MIT license
at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
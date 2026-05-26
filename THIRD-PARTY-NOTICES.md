# Third-Party Notices

This file records third-party components with additional redistribution notices
that are not obvious from the Rust source alone.

For source distributions, `Cargo.lock` records the full resolved Rust dependency
graph. For binary releases, this file highlights third-party components with
redistribution notices that are not obvious from the Rust source alone.


## Application Icons

The SVG icons under `assets/icons/` are exported from the application's
`IconKind` path data source. The path data is understood to be based on the
Material Design Icons icon set.

- Purpose: application UI icons.
- Source: https://pictogrammers.com/library/mdi/
- License: Apache License 2.0.
- Notice: the SVG files may have been renamed or regrouped for this project,
  but the underlying icon path data is from Material Design Icons.

## Opus Playback Support

The application enables `symphonia-adapter-libopus` with its `bundled` feature.
That feature enables `opusic-sys/bundled`, which builds the bundled libopus
source packaged inside `opusic-sys`.

### symphonia-adapter-libopus 0.3.0

- Purpose: adapter that registers libopus decoding with Symphonia.
- Source: https://crates.io/crates/symphonia-adapter-libopus
- Repository: https://github.com/pdeljanov/Symphonia
- License: MIT OR Apache-2.0.
- Notice choice for this project: MIT license terms below.

MIT License

Copyright (c) 2025 Austin Schey

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

### opusic-sys 0.7.3 and bundled libopus 1.6.1

- Purpose: Rust FFI bindings and build glue for libopus.
- Source: https://crates.io/crates/opusic-sys
- Repository: https://github.com/DoumanAsh/opusic-sys
- Cargo license metadata: BSD-3-Clause.
- Bundled native source: libopus 1.6.1, packaged under
  `opusic-sys-0.7.3/opus`.
- libopus source: https://opus-codec.org/

The `opusic-sys` package ships the following license text for the bundled Opus
code and applies it as the package license notice:

Copyright 2001-2023 Xiph.Org, Skype Limited, Octasic,
                    Jean-Marc Valin, Timothy B. Terriberry,
                    CSIRO, Gregory Maxwell, Mark Borgerding,
                    Erik de Castro Lopo, Mozilla, Amazon

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions
are met:

- Redistributions of source code must retain the above copyright
notice, this list of conditions and the following disclaimer.

- Redistributions in binary form must reproduce the above copyright
notice, this list of conditions and the following disclaimer in the
documentation and/or other materials provided with the distribution.

- Neither the name of Internet Society, IETF or IETF Trust, nor the
names of specific contributors, may be used to endorse or promote
products derived from this software without specific prior written
permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
``AS IS'' AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER
OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF
LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

Opus is subject to the royalty-free patent licenses which are
specified at:

Xiph.Org Foundation:
https://datatracker.ietf.org/ipr/1524/

Microsoft Corporation:
https://datatracker.ietf.org/ipr/1914/

Broadcom Corporation:
https://datatracker.ietf.org/ipr/1526/

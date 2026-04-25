# Model License Notices

## Release Artifact License Summary

- Release artifacts without `-with-model` do not bundle a Vaporetto model. They
  contain the `duckdb-vaporetto` extension, which is licensed under
  `MIT OR Apache-2.0`.
- Release artifacts with `-with-model` contain the `duckdb-vaporetto` extension
  under `MIT OR Apache-2.0` and additionally bundle the Vaporetto distribution
  model
  [`bccwj-suw+unidic_pos+kana.model.zst`](https://github.com/daac-tools/vaporetto-models/releases),
  which is licensed under
  [BSD-3-Clause](https://opensource.org/license/BSD-3-Clause).

## Bundled Model Notice

The `-with-model` release artifacts embed the Vaporetto distribution model
[`bccwj-suw+unidic_pos+kana.model.zst`](https://github.com/daac-tools/vaporetto-models/releases)
from `daac-tools/vaporetto-models` v0.5.0.

## bccwj-suw+unidic_pos+kana.model.zst

Copyright (c) 2011-2021, The UniDic Consortium
Copyright (c) 2022, LegalForce, Inc.
Copyright (c) 2023, LegalOn Technologies, Inc.
All rights reserved.

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are
met:

 * Redistributions of source code must retain the above copyright
   notice, this list of conditions and the following disclaimer.

 * Redistributions in binary form must reproduce the above copyright
   notice, this list of conditions and the following disclaimer in the
   documentation and/or other materials provided with the
   distribution.

 * Neither the name of the UniDic Consortium nor the names of its
   contributors may be used to endorse or promote products derived
   from this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
"AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
(INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

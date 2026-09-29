# 第三方许可证与版权声明

本项目 **t2048**（Cargo 包名 `t2048`）以 MIT 许可证发布，详见仓库根目录的 [`LICENSE`](LICENSE)。

t2048 编译成**单个可执行文件**分发：crates.io 上的源码包，以及 [GitHub Releases](https://github.com/microfatrat/t2048/releases) 里的 Linux / Windows / macOS 预编译二进制。运行时没有额外依赖，但可执行文件里**静态链接**了下列第三方组件，MIT / Apache-2.0 / BSD 等许可证都要求在再分发时保留它们的版权声明与许可证全文。

> **本文件由脚本自动生成，请勿手工编辑。**
> 生成命令：`python3 scripts/gen-third-party-licenses.py`
> 数据来源：`Cargo.lock` 与本地 Cargo 注册表缓存。
> 覆盖范围：`Cargo.lock` 中 97 个包，其中 96 个已解析许可证。

## 1. 本项目自身

- 许可证：MIT
- 版权：Copyright (c) 2026 microfatrat
- 全文见 [`LICENSE`](LICENSE)

## 2. 直接依赖

| 组件 | 版本 | 许可证 |
| --- | --- | --- |
| `crossterm` | 0.29.0 | `MIT` |
| `rand` | 0.10.3 | `MIT OR Apache-2.0` |
| `ratatui` | 0.30.2 | `MIT` |

## 3. 许可证全文

下列许可证文本按 SPDX 标识符去重，覆盖本次生成时能解析到的全部组件。当组件采用「A OR B」这类多许可表达式时，本项目选择其中限制最少的一项（通常是 MIT 或 Apache-2.0）。

各组件声明的许可证标识符（按组件数）：`MIT`（93）、`Apache-2.0`（73）、`Apache-2.0 WITH LLVM-exception`（3）、`Zlib`（1）、`LGPL-2.1-or-later`（1）、`BSL-1.0`（1）、`Unicode-3.0`（1）。

### 3.1 MIT License（`MIT`）

<!-- 取自 allocator-api2 0.2.21 的 LICENSE-MIT -->

```text
Permission is hereby granted, free of charge, to any
person obtaining a copy of this software and associated
documentation files (the "Software"), to deal in the
Software without restriction, including without
limitation the rights to use, copy, modify, merge,
publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software
is furnished to do so, subject to the following
conditions:

The above copyright notice and this permission notice
shall be included in all copies or substantial portions
of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.
```

### 3.2 Apache License 2.0（`Apache-2.0`）

<!-- 取自 allocator-api2 0.2.21 的 LICENSE-APACHE -->

```text
Apache License
                        Version 2.0, January 2004
                     http://www.apache.org/licenses/

TERMS AND CONDITIONS FOR USE, REPRODUCTION, AND DISTRIBUTION

1. Definitions.

   "License" shall mean the terms and conditions for use, reproduction,
   and distribution as defined by Sections 1 through 9 of this document.

   "Licensor" shall mean the copyright owner or entity authorized by
   the copyright owner that is granting the License.

   "Legal Entity" shall mean the union of the acting entity and all
   other entities that control, are controlled by, or are under common
   control with that entity. For the purposes of this definition,
   "control" means (i) the power, direct or indirect, to cause the
   direction or management of such entity, whether by contract or
   otherwise, or (ii) ownership of fifty percent (50%) or more of the
   outstanding shares, or (iii) beneficial ownership of such entity.

   "You" (or "Your") shall mean an individual or Legal Entity
   exercising permissions granted by this License.

   "Source" form shall mean the preferred form for making modifications,
   including but not limited to software source code, documentation
   source, and configuration files.

   "Object" form shall mean any form resulting from mechanical
   transformation or translation of a Source form, including but
   not limited to compiled object code, generated documentation,
   and conversions to other media types.

   "Work" shall mean the work of authorship, whether in Source or
   Object form, made available under the License, as indicated by a
   copyright notice that is included in or attached to the work
   (an example is provided in the Appendix below).

   "Derivative Works" shall mean any work, whether in Source or Object
   form, that is based on (or derived from) the Work and for which the
   editorial revisions, annotations, elaborations, or other modifications
   represent, as a whole, an original work of authorship. For the purposes
   of this License, Derivative Works shall not include works that remain
   separable from, or merely link (or bind by name) to the interfaces of,
   the Work and Derivative Works thereof.

   "Contribution" shall mean any work of authorship, including
   the original version of the Work and any modifications or additions
   to that Work or Derivative Works thereof, that is intentionally
   submitted to Licensor for inclusion in the Work by the copyright owner
   or by an individual or Legal Entity authorized to submit on behalf of
   the copyright owner. For the purposes of this definition, "submitted"
   means any form of electronic, verbal, or written communication sent
   to the Licensor or its representatives, including but not limited to
   communication on electronic mailing lists, source code control systems,
   and issue tracking systems that are managed by, or on behalf of, the
   Licensor for the purpose of discussing and improving the Work, but
   excluding communication that is conspicuously marked or otherwise
   designated in writing by the copyright owner as "Not a Contribution."

   "Contributor" shall mean Licensor and any individual or Legal Entity
   on behalf of whom a Contribution has been received by Licensor and
   subsequently incorporated within the Work.

2. Grant of Copyright License. Subject to the terms and conditions of
   this License, each Contributor hereby grants to You a perpetual,
   worldwide, non-exclusive, no-charge, royalty-free, irrevocable
   copyright license to reproduce, prepare Derivative Works of,
   publicly display, publicly perform, sublicense, and distribute the
   Work and such Derivative Works in Source or Object form.

3. Grant of Patent License. Subject to the terms and conditions of
   this License, each Contributor hereby grants to You a perpetual,
   worldwide, non-exclusive, no-charge, royalty-free, irrevocable
   (except as stated in this section) patent license to make, have made,
   use, offer to sell, sell, import, and otherwise transfer the Work,
   where such license applies only to those patent claims licensable
   by such Contributor that are necessarily infringed by their
   Contribution(s) alone or by combination of their Contribution(s)
   with the Work to which such Contribution(s) was submitted. If You
   institute patent litigation against any entity (including a
   cross-claim or counterclaim in a lawsuit) alleging that the Work
   or a Contribution incorporated within the Work constitutes direct
   or contributory patent infringement, then any patent licenses
   granted to You under this License for that Work shall terminate
   as of the date such litigation is filed.

4. Redistribution. You may reproduce and distribute copies of the
   Work or Derivative Works thereof in any medium, with or without
   modifications, and in Source or Object form, provided that You
   meet the following conditions:

   (a) You must give any other recipients of the Work or
       Derivative Works a copy of this License; and

   (b) You must cause any modified files to carry prominent notices
       stating that You changed the files; and

   (c) You must retain, in the Source form of any Derivative Works
       that You distribute, all copyright, patent, trademark, and
       attribution notices from the Source form of the Work,
       excluding those notices that do not pertain to any part of
       the Derivative Works; and

   (d) If the Work includes a "NOTICE" text file as part of its
       distribution, then any Derivative Works that You distribute must
       include a readable copy of the attribution notices contained
       within such NOTICE file, excluding those notices that do not
       pertain to any part of the Derivative Works, in at least one
       of the following places: within a NOTICE text file distributed
       as part of the Derivative Works; within the Source form or
       documentation, if provided along with the Derivative Works; or,
       within a display generated by the Derivative Works, if and
       wherever such third-party notices normally appear. The contents
       of the NOTICE file are for informational purposes only and
       do not modify the License. You may add Your own attribution
       notices within Derivative Works that You distribute, alongside
       or as an addendum to the NOTICE text from the Work, provided
       that such additional attribution notices cannot be construed
       as modifying the License.

   You may add Your own copyright statement to Your modifications and
   may provide additional or different license terms and conditions
   for use, reproduction, or distribution of Your modifications, or
   for any such Derivative Works as a whole, provided Your use,
   reproduction, and distribution of the Work otherwise complies with
   the conditions stated in this License.

5. Submission of Contributions. Unless You explicitly state otherwise,
   any Contribution intentionally submitted for inclusion in the Work
   by You to the Licensor shall be under the terms and conditions of
   this License, without any additional terms or conditions.
   Notwithstanding the above, nothing herein shall supersede or modify
   the terms of any separate license agreement you may have executed
   with Licensor regarding such Contributions.

6. Trademarks. This License does not grant permission to use the trade
   names, trademarks, service marks, or product names of the Licensor,
   except as required for reasonable and customary use in describing the
   origin of the Work and reproducing the content of the NOTICE file.

7. Disclaimer of Warranty. Unless required by applicable law or
   agreed to in writing, Licensor provides the Work (and each
   Contributor provides its Contributions) on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
   implied, including, without limitation, any warranties or conditions
   of TITLE, NON-INFRINGEMENT, MERCHANTABILITY, or FITNESS FOR A
   PARTICULAR PURPOSE. You are solely responsible for determining the
   appropriateness of using or redistributing the Work and assume any
   risks associated with Your exercise of permissions under this License.

8. Limitation of Liability. In no event and under no legal theory,
   whether in tort (including negligence), contract, or otherwise,
   unless required by applicable law (such as deliberate and grossly
   negligent acts) or agreed to in writing, shall any Contributor be
   liable to You for damages, including any direct, indirect, special,
   incidental, or consequential damages of any character arising as a
   result of this License or out of the use or inability to use the
   Work (including but not limited to damages for loss of goodwill,
   work stoppage, computer failure or malfunction, or any and all
   other commercial damages or losses), even if such Contributor
   has been advised of the possibility of such damages.

9. Accepting Warranty or Additional Liability. While redistributing
   the Work or Derivative Works thereof, You may choose to offer,
   and charge a fee for, acceptance of support, warranty, indemnity,
   or other liability obligations and/or rights consistent with this
   License. However, in accepting such obligations, You may act only
   on Your own behalf and on Your sole responsibility, not on behalf
   of any other Contributor, and only if You agree to indemnify,
   defend, and hold each Contributor harmless for any liability
   incurred by, or claims asserted against, such Contributor by reason
   of your accepting any such warranty or additional liability.

END OF TERMS AND CONDITIONS
```

### 3.3 zlib License（`Zlib`）

<!-- 取自 foldhash 0.2.0 的 LICENSE -->

```text
Copyright (c) 2024 Orson Peters

This software is provided 'as-is', without any express or implied warranty. In
no event will the authors be held liable for any damages arising from the use of
this software.

Permission is granted to anyone to use this software for any purpose, including
commercial applications, and to alter it and redistribute it freely, subject to
the following restrictions:

1. The origin of this software must not be misrepresented; you must not claim
    that you wrote the original software. If you use this software in a product,
    an acknowledgment in the product documentation would be appreciated but is
    not required.

2. Altered source versions must be plainly marked as such, and must not be
    misrepresented as being the original software.

3. This notice may not be removed or altered from any source distribution.
```

### 3.4 UNICODE LICENSE V3（`Unicode-3.0`）

<!-- 取自 unicode-ident 1.0.26 的 LICENSE-UNICODE -->

```text
UNICODE LICENSE V3

COPYRIGHT AND PERMISSION NOTICE

Copyright © 1991-2023 Unicode, Inc.

NOTICE TO USER: Carefully read the following legal agreement. BY
DOWNLOADING, INSTALLING, COPYING OR OTHERWISE USING DATA FILES, AND/OR
SOFTWARE, YOU UNEQUIVOCALLY ACCEPT, AND AGREE TO BE BOUND BY, ALL OF THE
TERMS AND CONDITIONS OF THIS AGREEMENT. IF YOU DO NOT AGREE, DO NOT
DOWNLOAD, INSTALL, COPY, DISTRIBUTE OR USE THE DATA FILES OR SOFTWARE.

Permission is hereby granted, free of charge, to any person obtaining a
copy of data files and any associated documentation (the "Data Files") or
software and any associated documentation (the "Software") to deal in the
Data Files or Software without restriction, including without limitation
the rights to use, copy, modify, merge, publish, distribute, and/or sell
copies of the Data Files or Software, and to permit persons to whom the
Data Files or Software are furnished to do so, provided that either (a)
this copyright and permission notice appear with all copies of the Data
Files or Software, or (b) this copyright and permission notice appear in
associated Documentation.

THE DATA FILES AND SOFTWARE ARE PROVIDED "AS IS", WITHOUT WARRANTY OF ANY
KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT OF
THIRD PARTY RIGHTS.

IN NO EVENT SHALL THE COPYRIGHT HOLDER OR HOLDERS INCLUDED IN THIS NOTICE
BE LIABLE FOR ANY CLAIM, OR ANY SPECIAL INDIRECT OR CONSEQUENTIAL DAMAGES,
OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS,
WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION,
ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THE DATA
FILES OR SOFTWARE.

Except as contained in this notice, the name of a copyright holder shall
not be used in advertising or otherwise to promote the sale, use or other
dealings in these Data Files or Software without prior written
authorization of the copyright holder.
```

### 3.5 Boost Software License 1.0（`BSL-1.0`）

<!-- 取自 ryu 1.0.23 的 LICENSE-BOOST -->

```text
Boost Software License - Version 1.0 - August 17th, 2003

Permission is hereby granted, free of charge, to any person or organization
obtaining a copy of the software and accompanying documentation covered by
this license (the "Software") to use, reproduce, display, distribute,
execute, and transmit the Software, and to prepare derivative works of the
Software, and to permit third-parties to whom the Software is furnished to
do so, all subject to the following:

The copyright notices in the Software and this entire statement, including
the above license grant, this restriction and the following disclaimer,
must be included in all copies of the Software, in whole or in part, and
all derivative works of the Software, unless such copies or derivative
works are solely in the form of machine-executable object code generated by
a source language processor.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE, TITLE AND NON-INFRINGEMENT. IN NO EVENT
SHALL THE COPYRIGHT HOLDERS OR ANYONE DISTRIBUTING THE SOFTWARE BE LIABLE
FOR ANY DAMAGES OR OTHER LIABILITY, WHETHER IN CONTRACT, TORT OR OTHERWISE,
ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.
```

## 4. 全部第三方组件清单

共 96 个组件。许可证列是该 crate 在 `Cargo.toml` 中声明的 SPDX 表达式原文。

| # | 组件 | 版本 | 许可证 |
| --- | --- | --- | --- |
| 1 | `allocator-api2` | 0.2.21 | `MIT OR Apache-2.0` |
| 2 | `approx` | 0.5.1 | `Apache-2.0` |
| 3 | `autocfg` | 1.5.1 | `Apache-2.0 OR MIT` |
| 4 | `bitflags` | 2.13.2 | `MIT OR Apache-2.0` |
| 5 | `by_address` | 1.2.1 | `MIT OR Apache-2.0` |
| 6 | `castaway` | 0.2.4 | `MIT` |
| 7 | `cfg-if` | 1.0.5 | `MIT OR Apache-2.0` |
| 8 | `chacha20` | 0.10.2 | `MIT OR Apache-2.0` |
| 9 | `compact_str` | 0.9.1 | `MIT` |
| 10 | `convert_case` | 0.10.0 | `MIT` |
| 11 | `cpufeatures` | 0.3.1 | `MIT OR Apache-2.0` |
| 12 | `crossterm` | 0.29.0 | `MIT` |
| 13 | `crossterm_winapi` | 0.9.1 | `MIT` |
| 14 | `darling` | 0.24.1 | `MIT` |
| 15 | `darling_core` | 0.24.1 | `MIT` |
| 16 | `darling_macro` | 0.24.1 | `MIT` |
| 17 | `deranged` | 0.5.8 | `MIT OR Apache-2.0` |
| 18 | `derive_more` | 2.1.1 | `MIT` |
| 19 | `derive_more-impl` | 2.1.1 | `MIT` |
| 20 | `document-features` | 0.2.12 | `MIT OR Apache-2.0` |
| 21 | `either` | 1.18.0 | `MIT OR Apache-2.0` |
| 22 | `equivalent` | 1.0.2 | `Apache-2.0 OR MIT` |
| 23 | `errno` | 0.3.14 | `MIT OR Apache-2.0` |
| 24 | `foldhash` | 0.2.0 | `Zlib` |
| 25 | `getrandom` | 0.4.3 | `MIT OR Apache-2.0` |
| 26 | `hashbrown` | 0.16.1 | `MIT OR Apache-2.0` |
| 27 | `hashbrown` | 0.17.1 | `MIT OR Apache-2.0` |
| 28 | `heck` | 0.5.0 | `MIT OR Apache-2.0` |
| 29 | `ident_case` | 1.0.1 | `MIT/Apache-2.0` |
| 30 | `indoc` | 2.0.7 | `MIT OR Apache-2.0` |
| 31 | `instability` | 0.3.13 | `MIT` |
| 32 | `itertools` | 0.14.0 | `MIT OR Apache-2.0` |
| 33 | `itoa` | 1.0.18 | `MIT OR Apache-2.0` |
| 34 | `kasuari` | 0.4.12 | `MIT OR Apache-2.0` |
| 35 | `libc` | 0.2.189 | `MIT OR Apache-2.0` |
| 36 | `libm` | 0.2.16 | `MIT` |
| 37 | `line-clipping` | 0.3.8 | `MIT OR Apache-2.0` |
| 38 | `linux-raw-sys` | 0.12.1 | `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT` |
| 39 | `litrs` | 1.0.0 | `MIT OR Apache-2.0` |
| 40 | `lock_api` | 0.4.14 | `MIT OR Apache-2.0` |
| 41 | `log` | 0.4.34 | `MIT OR Apache-2.0` |
| 42 | `lru` | 0.18.4 | `MIT` |
| 43 | `mio` | 1.2.3 | `MIT` |
| 44 | `num-conv` | 0.2.2 | `MIT OR Apache-2.0` |
| 45 | `num-traits` | 0.2.19 | `MIT OR Apache-2.0` |
| 46 | `num_threads` | 0.1.7 | `MIT OR Apache-2.0` |
| 47 | `palette` | 0.7.7 | `MIT OR Apache-2.0` |
| 48 | `palette_derive` | 0.7.7 | `MIT OR Apache-2.0` |
| 49 | `palette_math` | 0.7.7 | `MIT OR Apache-2.0` |
| 50 | `parking_lot` | 0.12.5 | `MIT OR Apache-2.0` |
| 51 | `parking_lot_core` | 0.9.12 | `MIT OR Apache-2.0` |
| 52 | `portable-atomic` | 1.15.0 | `Apache-2.0 OR MIT` |
| 53 | `powerfmt` | 0.2.0 | `MIT OR Apache-2.0` |
| 54 | `proc-macro2` | 1.0.107 | `MIT OR Apache-2.0` |
| 55 | `quote` | 1.0.47 | `MIT OR Apache-2.0` |
| 56 | `r-efi` | 6.0.0 | `MIT OR Apache-2.0 OR LGPL-2.1-or-later` |
| 57 | `rand` | 0.10.3 | `MIT OR Apache-2.0` |
| 58 | `rand_core` | 0.10.1 | `MIT OR Apache-2.0` |
| 59 | `ratatui` | 0.30.2 | `MIT` |
| 60 | `ratatui-core` | 0.1.2 | `MIT` |
| 61 | `ratatui-crossterm` | 0.1.2 | `MIT` |
| 62 | `ratatui-widgets` | 0.3.2 | `MIT` |
| 63 | `redox_syscall` | 0.5.18 | `MIT` |
| 64 | `rustc_version` | 0.4.1 | `MIT OR Apache-2.0` |
| 65 | `rustix` | 1.1.5 | `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT` |
| 66 | `rustversion` | 1.0.23 | `MIT OR Apache-2.0` |
| 67 | `ryu` | 1.0.23 | `Apache-2.0 OR BSL-1.0` |
| 68 | `scopeguard` | 1.2.0 | `MIT OR Apache-2.0` |
| 69 | `semver` | 1.0.28 | `MIT OR Apache-2.0` |
| 70 | `serde` | 1.0.229 | `MIT OR Apache-2.0` |
| 71 | `serde_core` | 1.0.229 | `MIT OR Apache-2.0` |
| 72 | `serde_derive` | 1.0.229 | `MIT OR Apache-2.0` |
| 73 | `signal-hook` | 0.3.18 | `Apache-2.0/MIT` |
| 74 | `signal-hook-mio` | 0.2.5 | `MIT OR Apache-2.0` |
| 75 | `signal-hook-registry` | 1.4.8 | `MIT OR Apache-2.0` |
| 76 | `smallvec` | 1.16.1 | `MIT OR Apache-2.0` |
| 77 | `static_assertions` | 1.1.0 | `MIT OR Apache-2.0` |
| 78 | `strsim` | 0.11.1 | `MIT` |
| 79 | `strum` | 0.28.0 | `MIT` |
| 80 | `strum_macros` | 0.28.0 | `MIT` |
| 81 | `syn` | 2.0.119 | `MIT OR Apache-2.0` |
| 82 | `syn` | 3.0.6 | `MIT OR Apache-2.0` |
| 83 | `thiserror` | 2.0.20 | `MIT OR Apache-2.0` |
| 84 | `thiserror-impl` | 2.0.20 | `MIT OR Apache-2.0` |
| 85 | `time` | 0.3.55 | `MIT OR Apache-2.0` |
| 86 | `time-core` | 0.1.9 | `MIT OR Apache-2.0` |
| 87 | `unicode-ident` | 1.0.26 | `(MIT OR Apache-2.0) AND Unicode-3.0` |
| 88 | `unicode-segmentation` | 1.13.3 | `MIT OR Apache-2.0` |
| 89 | `unicode-truncate` | 2.0.1 | `MIT OR Apache-2.0` |
| 90 | `unicode-width` | 0.2.2 | `MIT OR Apache-2.0` |
| 91 | `wasi` | 0.11.1+wasi-snapshot-preview1 | `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT` |
| 92 | `winapi` | 0.3.9 | `MIT/Apache-2.0` |
| 93 | `winapi-i686-pc-windows-gnu` | 0.4.0 | `MIT/Apache-2.0` |
| 94 | `winapi-x86_64-pc-windows-gnu` | 0.4.0 | `MIT/Apache-2.0` |
| 95 | `windows-link` | 0.2.1 | `MIT OR Apache-2.0` |
| 96 | `windows-sys` | 0.61.2 | `MIT OR Apache-2.0` |

## 5. 版权声明明细

以下版权行摘自各组件自带的许可证文件。MIT / BSD / ISC / zlib 等许可证都要求在分发时保留这些声明。

- **autocfg 1.5.1** — Copyright (c) 2018 Josh Stone
- **bitflags 2.13.2** — Copyright (c) 2014 The Rust Project Developers
- **by_address 1.2.1** — Copyright (c) 2017 Matt Brubeck
- **castaway 0.2.4** — Copyright (c) 2021 Stephen M. Coakley
- **cfg-if 1.0.5** — Copyright (c) 2014 Alex Crichton
- **chacha20 0.10.2** — Copyright (c) 2019-2026 The RustCrypto Project Developers
- **compact_str 0.9.1** — Copyright (c) 2021 Parker Timmerman
- **convert_case 0.10.0** — Copyright (c) 2025 rutrum
- **cpufeatures 0.3.1** — Copyright (c) 2020-2026 The RustCrypto Project Developers
- **crossterm 0.29.0** — Copyright (c) 2019 Timon
- **crossterm_winapi 0.9.1** — Copyright (c) 2019 Timon
- **darling 0.24.1** — Copyright (c) 2017 Ted Driggs
- **darling_core 0.24.1** — Copyright (c) 2017 Ted Driggs
- **darling_macro 0.24.1** — Copyright (c) 2017 Ted Driggs
- **deranged 0.5.8** — Copyright 2024 Jacob Pratt et al.；Copyright (c) 2024 Jacob Pratt et al.
- **derive_more 2.1.1** — Copyright (c) 2016 Jelte Fennema
- **derive_more-impl 2.1.1** — Copyright (c) 2016 Jelte Fennema
- **document-features 0.2.12** — Copyright (c) 2020 Olivier Goffart <ogoffart@sixtyfps.io>
- **either 1.18.0** — Copyright (c) 2015
- **equivalent 1.0.2** — Copyright (c) 2016--2023
- **errno 0.3.14** — Copyright (c) 2014 Chris Wong
- **foldhash 0.2.0** — Copyright (c) 2024 Orson Peters
- **getrandom 0.4.3** — Copyright (c) 2018-2026 The rust-random Project Developers；Copyright (c) 2014 The Rust Project Developers
- **hashbrown 0.16.1** — Copyright (c) 2016 Amanieu d'Antras
- **hashbrown 0.17.1** — Copyright (c) 2016 Amanieu d'Antras
- **heck 0.5.0** — Copyright (c) 2015 The Rust Project Developers
- **instability 0.3.13** — Copyright (c) 2020 Stephen M. Coakley；Copyright (c) The Ratatui Developers
- **itertools 0.14.0** — Copyright (c) 2015
- **kasuari 0.4.12** — Copyright (c) 2016 Dylan Ede；Copyright (c) 2024 Josh McKinney
- **libc 0.2.189** — Copyright (c) The Rust Project Developers
- **libm 0.2.16** — Copyright (c) 2018 Jorge Aparicio；Copyright © 2005-2020 Rich Felker, et al.；Copyright © 1993,2004 Sun Microsystems or；Copyright © 2003-2011 David Schultz or；Copyright © 2003-2009 Steven G. Kargl or；Copyright © 2003-2009 Bruce D. Evans or
- **line-clipping 0.3.8** — Copyright 2024 Josh McKinney；Copyright (c) 2024 Josh McKinney
- **litrs 1.0.0** — Copyright (c) 2020 Project Developers
- **lock_api 0.4.14** — Copyright (c) 2016 The Rust Project Developers
- **log 0.4.34** — Copyright (c) 2014 The Rust Project Developers
- **lru 0.18.4** — Copyright (c) 2016 Jerome Froelich
- **mio 1.2.3** — Copyright (c) 2014 Carl Lerche and other MIO contributors
- **num-conv 0.2.2** — Copyright (c) Jacob Pratt
- **num-traits 0.2.19** — Copyright (c) 2014 The Rust Project Developers
- **num_threads 0.1.7** — Copyright 2021 Jacob Pratt；Copyright (c) 2021 Jacob Pratt
- **palette 0.7.7** — Copyright (c) 2015 Erik Hedvall
- **palette_derive 0.7.7** — Copyright (c) 2015 Erik Hedvall
- **palette_math 0.7.7** — Copyright (c) 2015 Erik Hedvall
- **parking_lot 0.12.5** — Copyright (c) 2016 The Rust Project Developers
- **parking_lot_core 0.9.12** — Copyright (c) 2016 The Rust Project Developers
- **powerfmt 0.2.0** — Copyright 2023 Jacob Pratt et al.；Copyright (c) 2023 Jacob Pratt et al.
- **rand 0.10.3** — Copyright 2018 Developers of the Rand project；Copyright (c) 2014 The Rust Project Developers
- **rand_core 0.10.1** — Copyright (c) 2018-2026 The Rand Project Developers
- **ratatui 0.30.2** — Copyright (c) 2016-2022 Florian Dehau；Copyright (c) 2023-2025 The Ratatui Developers
- **ratatui-core 0.1.2** — Copyright (c) 2016-2022 Florian Dehau；Copyright (c) 2023-2025 The Ratatui Developers
- **ratatui-crossterm 0.1.2** — Copyright (c) 2016-2022 Florian Dehau；Copyright (c) 2023-2025 The Ratatui Developers
- **ratatui-widgets 0.3.2** — Copyright (c) 2016-2022 Florian Dehau；Copyright (c) 2023-2025 The Ratatui Developers
- **redox_syscall 0.5.18** — Copyright (c) 2017 Redox OS Developers
- **rustc_version 0.4.1** — Copyright (c) 2016 The Rust Project Developers
- **scopeguard 1.2.0** — Copyright (c) 2016-2019 Ulrik Sverdrup "bluss" and scopeguard developers
- **signal-hook 0.3.18** — Copyright (c) 2017 tokio-jsonrpc developers
- **signal-hook-mio 0.2.5** — Copyright (c) 2017 tokio-jsonrpc developers
- **signal-hook-registry 1.4.8** — Copyright (c) 2017 tokio-jsonrpc developers
- **smallvec 1.16.1** — Copyright (c) 2018 The Servo Project Developers
- **static_assertions 1.1.0** — Copyright (c) 2017 Nikolai Vazquez
- **strsim 0.11.1** — Copyright (c) 2015 Danny Guo；Copyright (c) 2016 Titus Wormer <tituswormer@gmail.com>；Copyright (c) 2018 Akash Kurdekar
- **strum 0.28.0** — Copyright (c) 2019 Peter Glotfelty
- **strum_macros 0.28.0** — Copyright (c) 2019 Peter Glotfelty
- **time 0.3.55** — Copyright (c) Jacob Pratt et al.
- **time-core 0.1.9** — Copyright (c) Jacob Pratt et al.
- **unicode-ident 1.0.26** — Copyright © 1991-2023 Unicode, Inc.
- **unicode-segmentation 1.13.3** — Copyright (c) 2015 The Rust Project Developers
- **unicode-truncate 2.0.1** — Copyright (c) 2019 Aetf <aetf at unlimitedcodeworks dot xyz>
- **unicode-width 0.2.2** — Copyright (c) 2015 The Rust Project Developers
- **winapi 0.3.9** — Copyright (c) 2015-2018 The winapi-rs Developers
- **windows-link 0.2.1** — Copyright (c) Microsoft Corporation.
- **windows-sys 0.61.2** — Copyright (c) Microsoft Corporation.

## 6. 重新生成

```bash
# 拉取所有平台的依赖；不带 --target 时 cargo 会抓取全部目标三元组
cargo fetch --locked
# 重新生成
python3 scripts/gen-third-party-licenses.py
```

`--check` 只比较结果不写入，CI 用它确认清单没有随 `Cargo.lock` 过期。生成结果取决于本机 Cargo 缓存里存在哪些平台的 crate，因此 `--check` 要在与生成时相同的缓存状态下运行（两者都执行过完整的 `cargo fetch --locked`）。注册表缓存目录不可写时，可把 `CARGO_HOME` 指向一个可写目录再执行上面的命令。

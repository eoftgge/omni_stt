# Third-Party Notices

Omni-STT is distributed with the following third-party components.

## Fonts

The fonts below are embedded in the Omni-STT executable and are licensed under
the SIL Open Font License, Version 1.1.

    NotoSans-Medium.ttf
    NotoSansArabic-Medium.ttf
    NotoSansGunjalaGondi-Medium.ttf

        Copyright 2022 The Noto Project Authors

    NotoSansJP-Medium.ttf
    NotoSansKR-Medium.ttf
    NotoSansSC-Medium.ttf
    NotoSansTC-Medium.ttf

        Copyright (c) 2014-2021 Adobe (http://www.adobe.com/),
        with Reserved Font Name 'Source'.

The full text of the SIL Open Font License, Version 1.1 follows.
-----------------------------------------------------------
SIL OPEN FONT LICENSE Version 1.1 - 26 February 2007
-----------------------------------------------------------

PREAMBLE
The goals of the Open Font License (OFL) are to stimulate worldwide
development of collaborative font projects, to support the font creation
efforts of academic and linguistic communities, and to provide a free and
open framework in which fonts may be shared and improved in partnership
with others.

The OFL allows the licensed fonts to be used, studied, modified and
redistributed freely as long as they are not sold by themselves. The
fonts, including any derivative works, can be bundled, embedded,
redistributed and/or sold with any software provided that any reserved
names are not used by derivative works. The fonts and derivatives,
however, cannot be released under any other type of license. The
requirement for fonts to remain under this license does not apply
to any document created using the fonts or their derivatives.

DEFINITIONS
"Font Software" refers to the set of files released by the Copyright
Holder(s) under this license and clearly marked as such. This may
include source files, build scripts and documentation.

"Reserved Font Name" refers to any names specified as such after the
copyright statement(s).

"Original Version" refers to the collection of Font Software components as
distributed by the Copyright Holder(s).

"Modified Version" refers to any derivative made by adding to, deleting,
or substituting -- in part or in whole -- any of the components of the
Original Version, by changing formats or by porting the Font Software to a
new environment.

"Author" refers to any designer, engineer, programmer, technical
writer or other person who contributed to the Font Software.

PERMISSION & CONDITIONS
Permission is hereby granted, free of charge, to any person obtaining
a copy of the Font Software, to use, study, copy, merge, embed, modify,
redistribute, and sell modified and unmodified copies of the Font
Software, subject to the following conditions:

1) Neither the Font Software nor any of its individual components,
   in Original or Modified Versions, may be sold by itself.

2) Original or Modified Versions of the Font Software may be bundled,
   redistributed and/or sold with any software, provided that each copy
   contains the above copyright notice and this license. These can be
   included either as stand-alone text files, human-readable headers or
   in the appropriate machine-readable metadata fields within text or
   binary files as long as those fields can be easily viewed by the user.

3) No Modified Version of the Font Software may use the Reserved Font
   Name(s) unless explicit written permission is granted by the corresponding
   Copyright Holder. This restriction only applies to the primary font name as
   presented to the users.

4) The name(s) of the Copyright Holder(s) or the Author(s) of the Font
   Software shall not be used to promote, endorse or advertise any
   Modified Version, except to acknowledge the contribution(s) of the
   Copyright Holder(s) and the Author(s) or with their explicit written
   permission.

5) The Font Software, modified or unmodified, in part or in whole,
   must be distributed entirely under this license, and must not be
   distributed under any other license. The requirement for fonts to
   remain under this license does not apply to any document created
   using the Font Software.

TERMINATION
This license becomes null and void if any of the above conditions are
not met.

DISCLAIMER
THE FONT SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO ANY WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT
OF COPYRIGHT, PATENT, TRADEMARK, OR OTHER RIGHT. IN NO EVENT SHALL THE
COPYRIGHT HOLDER BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,
INCLUDING ANY GENERAL, SPECIAL, INDIRECT, INCIDENTAL, OR CONSEQUENTIAL
DAMAGES, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
FROM, OUT OF THE USE OR INABILITY TO USE THE FONT SOFTWARE OR FROM
OTHER DEALINGS IN THE FONT SOFTWARE.

---

## Rust dependencies

(Apache-2.0 OR MIT) AND NCSA (1): libfuzzer-sys
(Apache-2.0 OR MIT) AND OFL-1.1 AND Ubuntu-font-1.0 (1): epaint_default_fonts
(Apache-2.0 OR MIT) AND Unicode-3.0 (1): unicode-ident
0BSD OR Apache-2.0 OR MIT (1): adler2
Apache-2.0 (15): ab_glyph, ab_glyph_rasterizer, accesskit_winit, codespan-reporting, cpal, gethostname, glutin, glutin_egl_sys, glutin_glx_sys, glutin_wgl_sys, openssl, owned_ttf_parser, spirv, unicode-general-category, winit
Apache-2.0 AND MIT (1): dpi
Apache-2.0 OR Apache-2.0 WITH LLVM-exception OR MIT (7): linux-raw-sys, linux-raw-sys, rustix, rustix, wasi, wasip2, wit-bindgen
Apache-2.0 OR BSD-2-Clause OR MIT (3): mach2, zerocopy, zerocopy-derive
Apache-2.0 OR BSD-3-Clause (2): moxcms, pxfm
Apache-2.0 OR BSD-3-Clause OR MIT (2): num_enum, num_enum_derive
Apache-2.0 OR CC0-1.0 (1): imgref
Apache-2.0 OR GPL-2.0 (1): self_cell
Apache-2.0 OR LGPL-2.1-or-later OR MIT (2): r-efi, r-efi
Apache-2.0 OR MIT (352): accesskit, accesskit_atspi_common, accesskit_consumer, accesskit_consumer, accesskit_consumer, accesskit_macos, accesskit_unix, accesskit_windows, aes, ahash, aligned, allocator-api2, alsa, android-activity, android_system_properties, anyhow, apple-native-keyring-store, arbitrary, arboard, arrayvec, as-raw-xcb-connection, as-slice, ash, async-broadcast, async-channel, async-executor, async-io, async-lock, async-process, async-recursion, async-signal, async-task, async-trait, atomic-waker, atspi, atspi-common, atspi-proxies, bit-set, bit-vec, bit_field, bitflags, bitflags, bitstream-io, block-buffer, block-padding, blocking, bumpalo, cbc, cfg-if, cgl, chacha20, cipher, cmov, color, concurrent-queue, const-oid, core-foundation, core-foundation, core-foundation-sys, core-graphics, core-graphics-types, coreaudio-rs, cpubits, cpufeatures, crc32fast, crossbeam-channel, crossbeam-deque, crossbeam-epoch, crossbeam-utils, crypto-common, ctutils, dasp_sample, deranged, digest, dirs, dirs-sys, displaydoc, document-features, downcast-rs, ecolor, eframe, egui, egui-wgpu, egui-winit, egui_glow, either, emath, embed_plist, enumflags2, enumflags2_derive, epaint, equivalent, errno, euclid, event-listener, event-listener-strategy, fastrand, fdeflate, fearless_simd, field-offset, flate2, font-types, foreign-types, foreign-types, foreign-types-macros, foreign-types-shared, foreign-types-shared, form_urlencoded, futures-channel, futures-core, futures-executor, futures-io, futures-lite, futures-macro, futures-sink, futures-task, futures-util, getrandom, getrandom, getrandom, gif, glifo, gpu-allocator, guillotiere, half, hashbrown, hashbrown, heck, hermit-abi, hex, hkdf, hmac, http, httparse, hybrid-array, idna, idna_adapter, image, image-webp, indexmap, inout, itertools, itertools, itoa, jni, jni-macros, jni-sys, jni-sys, jni-sys-macros, js-sys, keyboard-types, keyring, keyring-core, khronos-egl, kurbo, lazy_static, libappindicator, libappindicator-sys, libc, linebender_resource_handle, litrs, lock_api, log, memmap2, muda, naga, naga-types, native-tls, ndk, ndk-context, ndk-sys, no_std_io2, nohash-hasher, num, num-bigint, num-complex, num-conv, num-derive, num-integer, num-iter, num-rational, num-traits, once_cell, openssl-macros, openssl-probe, ordered-stream, parking, parking_lot, parking_lot_core, paste, pastey, peniko, percent-encoding, pin-project, pin-project-internal, pin-project-lite, piper, plain, png, polling, pollster, pollster, polycool, portable-atomic, portable-atomic-util, powerfmt, ppv-lite86, presser, proc-macro-crate, proc-macro-crate, proc-macro-crate, proc-macro-error, proc-macro-error-attr, proc-macro2, profiling, profiling-procmacros, qoi, quick-error, quote, rand, rand, rand_chacha, rand_core, rand_core, range-alloc, raw-window-metal, rayon, rayon-core, read-fonts, regex, regex-automata, regex-syntax, renderdoc-sys, rustc-hash, rustc-hash, rustversion, scoped-tls, scopeguard, secret-service, security-framework, security-framework-sys, serde, serde_core, serde_derive, serde_json, serde_repr, serde_spanned, serde_spanned, sha1, sha2, signal-hook-registry, simd_cesu8, simdutf8, siphasher, skrifa, smallvec, smol_str, socket2, stable_deref_trait, static_assertions, symlink, syn, syn, syn, tempfile, thiserror, thiserror, thiserror-impl, thiserror-impl, thread_local, time, time-core, time-macros, toml, toml_datetime, toml_datetime, toml_edit, toml_edit, toml_edit, toml_parser, toml_writer, tray-icon, ttf-parser, tungstenite, type-map, typenum, unicode-segmentation, unicode-width, url, utf8_iter, uuid, vello_common, vello_cpu, wasm-bindgen, wasm-bindgen-futures, wasm-bindgen-macro, wasm-bindgen-macro-support, wasm-bindgen-shared, web-sys, web-time, webbrowser, weezl, wgpu, wgpu-core, wgpu-core-deps-apple, wgpu-core-deps-emscripten, wgpu-core-deps-wasm, wgpu-core-deps-windows-linux-android, wgpu-hal, wgpu-naga-bridge, wgpu-types, winapi, winapi-i686-pc-windows-gnu, winapi-x86_64-pc-windows-gnu, windows, windows-collections, windows-core, windows-future, windows-implement, windows-interface, windows-link, windows-native-keyring-store, windows-numerics, windows-result, windows-strings, windows-sys, windows-sys, windows-sys, windows-sys, windows-targets, windows-targets, windows-threading, windows_aarch64_gnullvm, windows_aarch64_gnullvm, windows_aarch64_msvc, windows_aarch64_msvc, windows_i686_gnu, windows_i686_gnu, windows_i686_gnullvm, windows_i686_gnullvm, windows_i686_msvc, windows_i686_msvc, windows_x86_64_gnu, windows_x86_64_gnu, windows_x86_64_gnullvm, windows_x86_64_gnullvm, windows_x86_64_msvc, windows_x86_64_msvc, x11rb, x11rb-protocol, zbus-secret-service-keyring-store, zeroize
Apache-2.0 OR MIT OR Zlib (23): bytemuck, bytemuck_derive, cursor-icon, dispatch2, glow, miniz_oxide, miniz_oxide, objc2-app-kit, objc2-audio-toolbox, objc2-avf-audio, objc2-core-audio, objc2-core-audio-types, objc2-core-foundation, objc2-core-graphics, objc2-io-surface, objc2-metal, objc2-quartz-core, objc2-ui-kit, raw-window-handle, xkeysym, zune-core, zune-inflate, zune-jpeg
BSD-2-Clause (4): arrayref, av1-grain, rav1e, v_frame
BSD-3-Clause (6): avif-serialize, exr, lebe, ravif, tiny-skia, tiny-skia-path
BSL-1.0 (2): clipboard-win, error-code
ISC (2): libloading, libloading
MIT (149): aligned-vec, alsa-sys, android-properties, arg_enum_proc_macro, atk, atk-sys, av-scenechange, block2, block2, bytes, cairo-rs, cairo-sys-rs, calloop, calloop, calloop-wayland-source, calloop-wayland-source, color_quant, combine, crunchy, data-encoding, dispatch, dlib, egui-toast, endi, equator, equator-macro, fax, gdk, gdk-pixbuf, gdk-pixbuf-sys, gdk-sys, gio, gio-sys, glib, glib-macros, glib-sys, glutin-winit, gobject-sys, gtk, gtk-sys, gtk3-macros, harfrust, interpolate_name, libm, libredox, libxdo, libxdo-sys, loop9, maybe-rayon, memoffset, mio, new_debug_unreachable, nom, noop_proc_macro, nu-ansi-term, objc-sys, objc2, objc2, objc2-app-kit, objc2-cloud-kit, objc2-contacts, objc2-core-data, objc2-core-image, objc2-core-location, objc2-encode, objc2-foundation, objc2-foundation, objc2-link-presentation, objc2-metal, objc2-quartz-core, objc2-symbols, objc2-ui-kit, objc2-uniform-type-identifiers, objc2-user-notifications, openssl-sys, orbclient, ordered-float, pango, pango-sys, phf, phf_generator, phf_macros, phf_shared, pulp, pulp-wasm-simd-flag, quick-xml, raw-cpuid, reborrow, redox_syscall, redox_syscall, redox_syscall, redox_users, rfd, rgb, schannel, sctk-adwaita, sharded-slab, simd-adler32, simd_helpers, slab, smithay-client-toolkit, smithay-client-toolkit, smithay-clipboard, strict-num, synstructure, tiff, tokio, tokio-macros, tokio-native-tls, tokio-tungstenite, tokio-util, tracing, tracing-appender, tracing-attributes, tracing-core, tracing-log, tracing-subscriber, uds_windows, valuable, wayland-backend, wayland-client, wayland-csd-frame, wayland-cursor, wayland-protocols, wayland-protocols-experimental, wayland-protocols-misc, wayland-protocols-plasma, wayland-protocols-wlr, wayland-scanner, wayland-sys, winnow, winnow, x11, x11-dl, xcursor, xkbcommon-dl, y4m, zbus, zbus-lockstep, zbus-lockstep-macros, zbus_macros, zbus_names, zbus_xml, zcheapstr, zmij, zvariant, zvariant_derive, zvariant_utils
MIT OR Unlicense (6): aho-corasick, byteorder, byteorder-lite, memchr, termcolor, winapi-util
MPL-2.0 (1): option-ext
Unicode-3.0 (18): icu_collections, icu_locale_core, icu_normalizer, icu_normalizer_data, icu_properties, icu_properties_data, icu_provider, litemap, potential_utf, tinystr, writeable, yoke, yoke-derive, zerofrom, zerofrom-derive, zerotrie, zerovec, zerovec-derive
Zlib (3): foldhash, slotmap, zlib-rs

### egui bundled fonts (epaint_default_fonts)

- Hack-Regular.ttf — MIT, Copyright (c) 2018 Source Foundry Authors
- Ubuntu-Light.ttf — Ubuntu Font Licence 1.0, Copyright 2010–2011 Canonical Ltd.
- NotoEmoji-Regular.ttf — SIL OFL 1.1, Copyright The Noto Project Authors
- emoji-icon-font.ttf — MIT, Copyright (c) 2014 John Slegers
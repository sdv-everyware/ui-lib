# harry-app traps (all seen on the RB3)

| Symptom | Cause | Fix |
|---|---|---|
| every text breaks into single letters ("M O N", "0 7"), lines are clipped | the layout helper has no fonts: harry measures text with its own fonts, about 0 px wide | load every face into `HarLayoutHelperManager::load_font` too (one `FONTS` table) |
| a text sits slightly off | a weight of the design is missing; the nearest face stands in | add the real file (ask before downloading) |
| abort `Failed to make Impeller GL context current ... Permission denied (os error 13)` | another DRM master: the installed panel `smarthome` holds the display | `stop smarthome`, wait until no process has `card0` open |
| abort `couldn't find an OpenGL ES implementation` | the EGL loader cannot load Linaro's prebuilt mesa from `/vendor`: a library it needs has no `same_process_hal_file` label | the product's dragonboard fork labels it (`sepolicy/file_contexts`); check `logcat \| grep vndksupport` |
| a staircase of noise blocks | minigbm allocates compressed buffers (UBWC, modifier 0x0500000000000001), mesa renders uncompressed | `vendor.minigbm.debug=nocompression` (the property, not the variable; the product sets it): check `getprop vendor.minigbm.debug` |
| a touch does nothing, events with `x: 20700` | the panel reports 0..32767, harry took it as pixels | `--touch-size WxH`, which the app passes to `EvdevService::new_scaled` |
| a tap does not reach the app | the layer is hidden in Figma, renamed, or its field has no `interactive` | check the names in the `.dcf`; add `interactive` |
| part of the screen is below the display | the design is taller than the HDMI mode | ask the user: fill the stage, scale, or move it |
| about 7 frames a second, one thread at 50 % | harry's VERBOSE logs are formatted every frame | `log::set_max_level(Info)` in `main` and `setprop log.tag INFO` |
| half the CPU in the kernel (`clone`, `copy_page_range`) | `Command::new("getprop")` forks the big process | read the property with `__system_property_get` through FFI |
| 20 to 30 frames a second in animations, "tree rebuilt 30/30" | something structural changes every frame | `HAR_PASS_TIMING=1` logs which customization changed; position and opacity are display only, a text or a variant is not |
| build: `missing documentation for the crate` | `-D missing-docs` | a `//!` comment at the top of `main.rs` |
| build: `could not find Local in chrono` | Android's chrono has no clock feature | `SystemTime` from std |
| build: a module is not visible | the app is outside `packages/services/display_safety` | move it under `DS/utils/` |
| build: `env.sh` fails at `vendor-package-ver.sh` | sourced from zsh | run it in `bash -c '...'` |
| clippy: `unnecessary_map_or`, `large_enum_variant` | lints denied | `is_none_or`; `Box` the large variant |
| a fetched `.dcf` has the old layout | Figma's cache right after an edit | the fetch script's version check says so: wait, fetch again |
| harry's layout, text or display-list traces are needed | the app caps the log at INFO: Rust formats traces before liblog drops them | smarthome-app: run once with `SMARTHOME_LOG=trace` and `setprop log.tag VERBOSE`, `logcat -d` to a file, grep `Measured:` or `generate_dl` |
| no screenshot with `screencap` | no SurfaceFlinger on SDV Media | read the CRTC's framebuffer instead ([#20](https://github.com/sdv-everyware/everyware/issues/20)) |
| `adb shell` with `&` hangs | the background process keeps adb's channel open | `setsid nohup ... &`, or run the script in the background on the host |
| a text wraps on the panel but fits in Figma | Impeller's C API has no letter spacing: harry drops it, so a negative spacing draws the text wider than its box | spacing 0 in the design and the box sized for the font (everyware `docs/project/open-points.md`, 5) |
| an app in an APEX aborts at start with `Unable to find libEGL.so` | the APEX's linker namespace sees only the platform libraries the APEX declares; harry opens EGL with `dlopen` | link `libEGL` and `libGLESv2` (`shared_libs`, as smarthome-app's `Android.bp`): the build then declares them |

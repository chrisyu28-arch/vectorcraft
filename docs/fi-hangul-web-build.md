# Hangul fallback web build

This patch serves the unmodified Noto Sans CJK SC Regular font at startup through VectorCraft's existing web-font loader. The original v0.11.0 web build embeds BIZ UDPGothic and downloads Arabic and Thai faces, but does not load this CJK font. No font binary is added to this repository.

## Fixed inputs

- Upstream: [storytold/vectorcraft v0.11.0](https://github.com/storytold/vectorcraft/releases/tag/v0.11.0), commit `0a33cff3f6e2a1e9e672f4c3816f58cd8d3d032e`.
- Fonts: [storytold/craft-fonts](https://github.com/storytold/craft-fonts/tree/3d9ab3a132ed0eff457e424226a435a901da30d3), commit `3d9ab3a132ed0eff457e424226a435a901da30d3`.
- Exact face: `Noto Sans CJK SC | Regular`, `fonts/noto-sans-cjk-sc/NotoSansCJKsc-Regular.otf`.
- Font SHA-256: `2c76254f6fc379fddfce0a7e84fb5385bb135d3e399294f6eeb6680d0365b74b`.
- Font licence: SIL Open Font License 1.1; the original OFL text is packaged beside the font and at the package root. OFL SHA-256: `6a73f9541c2de74158c0e7cf6b0a58ef774f5a780bf191f2d7ec9cc53efe2bf2`.
- Original font: [notofonts/noto-cjk at f8d15753](https://github.com/notofonts/noto-cjk/blob/f8d157532fbfaeda587e826d4cd5b21a49186f7c/Sans/OTF/SimplifiedChinese/NotoSansCJKsc-Regular.otf).
- Toolchain: Rust 1.95.0, `wasm32-unknown-unknown`, Trunk 0.21.14, locked Cargo dependencies.

The family name is exact. `Noto Sans SC` is not the manifest entry used here, and its name alone would not prove Hangul coverage.

## Source compatibility with full CI

The upstream v0.11.0 workspace declares Rust 1.95 as its minimum version. Its Web release job selects the moving `stable` toolchain through the pinned rust-toolchain action; this workflow fixes 1.95.0 for repeatable tool selection, without claiming that every upstream release used that exact compiler. The original Web release job packages the app without running the full `cargo xtask ci` gate.

Running that gate on 1.95.0 exposed Clippy warnings in original upstream code. The renderer now expresses a missing or different cached pixel buffer with `is_none_or` and inequality, preserving both early-return cases. PDF import puts the existing shading-resource check in the match guard, still counting only readable shadings and ignoring absent ones. Layer-panel Boolean comparison uses inequality instead of equality with a negated operand. The thumbnail clock uses range containment with an explicit NaN exclusion, preserving negative, infinite and unordered elapsed-time behavior. The Photo patch similarly moves the CR3 preview-count, PNG missing-image, plugin-number range and testkit dependency checks into match guards, retaining the separate normal IEND and allowed dev-dependency arms. These equivalent source changes retain `-D warnings`; no lint or CI gate is disabled.

Both jobs first run Clippy with `--keep-going` to collect independent compiler failures in one run. This is an additional diagnostic pass with the same warning policy; the unchanged complete `cargo xtask ci` and mandatory font tests must still succeed afterward.

## Build and release

The dedicated `.github/workflows/fi-web-release.yml` runs only in `chrisyu28-arch/vectorcraft`.

1. Push the reviewed source to `codex/hangul-web`. Its job checks the official upstream tag and ancestry, the font commit and SHA-256, full upstream CI, the startup-font expectation and all 11,172 modern Hangul syllables (U+AC00–U+D7A3) using skrifa. A missing face fails when `CRAFT_FONTS_REQUIRED=1`; optional local builds without that font can skip.
2. The job applies the workspace version `0.11.0-fi.1`, builds the web package and verifies that its font bytes match the exact tested font SHA-256. The Trunk hook also checks the manifest hash and copies OFL. This establishes the packaged font's cmap result through identical bytes.
3. Review the successful build and its browser behaviour. The Actions artifact is named `vectorcraft-web-<source commit>` and retained for 30 days. It contains the ZIP, `SOURCE_PROVENANCE.json` and `SHA256SUMS.txt`.
4. Only after that review, push `v0.11.0-fi.1` at the exact tested source commit. The release job requires a successful push run of this workflow on `codex/hangul-web` with that commit, downloads its artifact, rechecks source/font/package hashes and publishes those same bytes as a prerelease. It does not rebuild or replace an existing release. A missing/expired artifact fails closed; rerun the branch build before retrying.

Provenance includes the public source repository and commit, upstream tag/commit, patch commit list, build run/attempt, version override, tool versions, Cargo.lock hash, font commit/hash and package hash. It contains no environment contents or credentials. Runner images and packaging timestamps can differ between separate builds; artifact promotion guarantees byte identity between a chosen successful build and its release, rather than claiming bit-for-bit rebuilds.

The original `.github/workflows/release.yml` only has `push.branches: [release]`; GitHub does not run it for this tag. The other upstream push workflows also restrict branches. Do not push the upstream `release` branch or manually dispatch its full desktop release pipeline for this web patch.

## Verification scope

The cmap guard checks nonzero glyph IDs for every modern precomposed Hangul syllable in the real font bytes. The workflow checks that the packaged bytes are identical and that licences are present. This is not a browser or IME test: Korean labels, filenames, document text, initial loading, reload and font-fetch failure still need browser verification. The existing startup loader retains its 20-second per-font timeout and warning/skip behaviour on failed downloads.

The patch does not change desktop font selection, translations, font outlines, source licensing, or hosting settings.


## PhotoCraft bundle from public source

The same workflow builds a second, independent job from [storytold/photocraft v0.9.0](https://github.com/storytold/photocraft/releases/tag/v0.9.0), commit `c699e3cda69f4852b43c0b191ab0091aaeff9d6a`, using its original craft-fonts commit `abb83316d96aa59c1cf64784289e378fe9fa5695`. It applies only the hash-pinned public source patch at `patches/fi/photocraft-hangul-v0.9.0.patch`; the patch carries PhotoCraft's own ASSETS/ATTRIBUTION documentation and the necessary loader, test and packaging changes. Original authorship and licence text remain intact. No third-party code-validation rule is changed to accommodate this patch.

The additional original font is downloaded at the fixed noto-cjk commit `f8d157532fbfaeda587e826d4cd5b21a49186f7c`. Its OTF bytes have the same font SHA above. Its licence is the original `Sans/LICENSE`: 4,301 bytes, SHA-256 `6a73f9541c2de74158c0e7cf6b0a58ef774f5a780bf191f2d7ec9cc53efe2bf2`, identical to the craft-fonts OFL bytes used by VectorCraft. Each download is checked independently without rewriting line endings or adding a final newline. The staged manifest is copied from PhotoCraft's patched `packaging/web/hangul-fonts-manifest.txt`.

Both jobs run their upstream `cargo xtask ci`, with clippy and native Linux build libraries installed, and debug information disabled for dev/test artifacts. Vector additionally runs its required font tests. Photo additionally runs `cargo test --locked -p photocraft-ui-egui hangul_font::tests -- --include-ignored --nocapture`, including the real-font SHA/cmap, document/menu registration, all egui font families and font reinstallation checks. Optional upstream corpus and visual-fixture tests retain their upstream behaviour; full CI is not evidence that those optional external fixtures were exercised.

Photo produces `photocraft-web-0.9.0-fi.1.zip`, `PHOTO_SOURCE_PROVENANCE.json` and `PHOTO_SHA256SUMS.txt`, in artifact `photocraft-web-<fork source commit>`. Its provenance binds the public fork commit, Photo upstream commit, patch SHA, original craft-fonts pin, noto-cjk pin, separate font/licence hashes, manifest hash and package hash. The original 24 MiB per-wasm limit remains in packaging and is rechecked against ZIP entries.

The tag job requires both named jobs to have succeeded in one branch workflow run at the exact tagged fork commit. It downloads both artifacts from that run, checks each provenance record against that run and commit, rechecks both sets of package/font/licence hashes, and publishes both ZIPs and both provenance/checksum files together under `v0.11.0-fi.1`. It never combines artifacts from different workflow runs or rebuilds on the tag.

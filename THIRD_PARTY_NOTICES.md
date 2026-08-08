# Third-party notices

`career-core` project-authored source is licensed under `MIT OR Apache-2.0`. The optional Swift binding adapter uses the following MPL-2.0 dependencies:

- UniFFI `0.30.0`
- `uniffi_bindgen` `0.30.0`
- `uniffi_core` `0.30.0`
- `uniffi_internal_macros` `0.30.0`
- `uniffi_macros` `0.30.0`
- `uniffi_meta` `0.30.0`
- `uniffi_pipeline` `0.30.0`
- `uniffi_udl` `0.30.0`

License: [Mozilla Public License 2.0](https://www.mozilla.org/MPL/2.0/)

Corresponding source: <https://github.com/mozilla/uniffi-rs/tree/v0.30.0>

UniFFI is used only by `crates/career-swift` and the pinned local binding-generation tool. Generated Swift and C declarations are produced from project-authored interface metadata by that version.

Any future distributed Swift/XCFramework artifact must include this notice, the project MIT/Apache licenses, and the source location above. No Swift artifact is currently published.

The Phase 9 npm launcher uses only Node built-in modules and adds no third-party Node dependency. Private tests and public candidate packages include this complete notice with both project licenses because native implementation packages contain the same locked Rust CLI. The protected npm release does not publish the Swift adapter or XCFramework.

Other Rust dependencies declare permissive license alternatives recorded in `Cargo.lock` and their crates.io package metadata. Release review must re-audit the exact locked dependency set before publication.

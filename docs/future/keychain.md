# Future: macOS Keychain integration

The goal is to move credential storage from the plaintext `~/.config/claude-usage-monitor/config.toml` (currently `chmod 600`) into the macOS Keychain, protected by Touch ID / user presence.

## What this would look like

- On first run (or credential update), the four values (org ID, session key, `cf_clearance`, `__cf_bm`) are written to Keychain as a generic password item using `kSecClassGenericPassword`, scoped to `kSecAttrAccessibleWhenUnlockedThisDeviceOnly` with no iCloud sync.
- On startup the app reads the item via `SecItemCopyMatching`; macOS automatically presents a Touch ID prompt if the access control policy requires it — no custom biometric code needed.
- The in-app setup screen (`e` key) and `--json` mode work the same way; only the storage backend changes.

## Implementation notes

- Use the [`security-framework`](https://crates.io/crates/security-framework) crate for Keychain operations.
- Biometric enforcement requires `SecAccessControlCreateWithFlags` with `kSecAccessControlBiometryCurrentSet` (or `kSecAccessControlUserPresence` as fallback). This may need a small FFI shim if the crate doesn't expose it directly.
- The `com.apple.quarantine` + Gatekeeper situation for unsigned binaries applies here too — the Keychain prompt will work fine for locally-built binaries.

## Current state

Not implemented. The `chmod 600` plaintext file is the "reasonably safe" floor: out of source control, not world-readable. PRs welcome.

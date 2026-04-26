Great, now we are going to stop storing the api keys in env, and instead we will add functionality in the TUI to set the keys, the keys will be stored from keychain and retrieved at app start. App needs option to add the keys, delete them, or change them. No need to store more than one set of keys

**Integration Note: Store Anthropic API Key in macOS Keychain with Touch ID Protection**

---

**Objective**
We are migrating storage of the Anthropic API key from plaintext (env/config) to the macOS Keychain, with biometric protection. The goal is to ensure the key is only retrievable after user authentication (Touch ID or passcode), without adding explicit authentication flows in the app.

---

**Approach**
Use the **macOS Keychain** to store the API key as a generic password item, configured with an access control policy that enforces biometric authentication.

We rely on **implicit authentication**:

- The app simply attempts to read the Keychain item
- macOS automatically prompts for Touch ID when required
- No direct interaction with biometric APIs is needed

---

**Implementation Plan**

### 1. Store the API key (setup path)

- On first run (or when key is missing):
  - Prompt user for their Anthropic API key
  - Store it in Keychain using:
    - `kSecClassGenericPassword`
    - A unique service name (e.g., `"com.<app>.anthropic"`)

  - Configure access control with:
    - `SecAccessControlCreateWithFlags`
    - Flags:
      - `kSecAccessControlBiometryCurrentSet` (preferred)
      - fallback: `kSecAccessControlUserPresence`

  - Set accessibility:
    - `kSecAttrAccessibleWhenUnlockedThisDeviceOnly`

**Important:**

- Do NOT store the key in config files, env vars, or logs
- Do NOT enable iCloud sync for this item

---

### 2. Retrieve the API key (runtime path)

- On app startup:
  - Call `SecItemCopyMatching` to fetch the Keychain item

- Expected behavior:
  - macOS automatically displays a Touch ID prompt
  - On success → API key is returned
  - On failure → handle error (exit or retry)

No manual use of LocalAuthentication APIs is required.

---

### 3. In-memory handling

- Once retrieved:
  - Store API key in memory only
  - Use it for API calls during the session

- Never persist it again outside Keychain

---

**Rust Integration Notes**

- Use the `security-framework` crate for basic Keychain operations
- For access control flags:
  - May require FFI (`SecAccessControlCreateWithFlags`) if not exposed

- Expect small amounts of `unsafe` code for:
  - CoreFoundation types
  - Access control configuration

---

**Behavior Summary**

- First run → user inputs API key → securely stored with biometric protection
- Subsequent runs → Keychain access triggers Touch ID automatically → key returned on success

---

**Non-Goals**

- Do not interact with Secure Enclave directly
- Do not implement custom biometric flows
- Do not expose the API key outside process memory

---

**Result**
The Anthropic API key is protected by system-level security and requires user presence (Touch ID) for access, with minimal changes to application logic.

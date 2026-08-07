# vodozemac-bindings

WASM (browser/Node) and React Native bindings for [vodozemac](https://github.com/matrix-org/vodozemac) — **Olm + Megolm** primitives for non-Matrix E2EE chat. No Matrix-specific types, no protocol assumptions about user/device id shape.

Publishes two npm packages with an identical JS surface (same class names, method names, and JSON-string return shapes), so an SDK can stay target-agnostic:

| Package | Target | How |
|---|---|---|
| `@kinsh/vodozemac-wasm` | Browser + Node | `wasm-bindgen` (`src/lib.rs`) |
| `@kinsh/vodozemac-rn` | React Native (new arch) | UniFFI → Swift/Kotlin + TurboModule (`crates/vodozemac-ffi`, `rn/`) |

Fork of [dTelecom/vodozemac-wasm](https://github.com/dTelecom/vodozemac-wasm) (Apache-2.0), which bound the Olm surface. This fork adds the Megolm group-session surface and republishes under the `@kinsh` scope.

## Why

`@matrix-org/olm` (libolm) has been EOL since Oct 2023 with known CVEs. The Matrix-recommended successor (`@matrix-org/matrix-sdk-crypto-wasm`) only exposes the high-level Matrix-protocol `OlmMachine`, not raw Olm/Megolm primitives. These packages fill the gap.

## Protocol versions

- **Olm**: `SessionConfig::version_1()` — libolm wire format, migration-friendly.
- **Megolm**: `SessionConfig::version_1()` — the format Matrix runs in production (AES-256 + HMAC truncated to 8 bytes). vodozemac's v2 is behind an experimental feature flag and intentionally not used.

Both targets pin the same versions; ciphertext and pickles interoperate across web/Node/RN.

## API surface — Olm (1:1)

```ts
import init, { Account, Session, InboundResult } from "@kinsh/vodozemac-wasm";
// or: from "@kinsh/vodozemac-rn" — identical surface, init() is a no-op.
await init();

const a = new Account();
a.identityKeys();                               // JSON { curve25519, ed25519 }
a.generateOneTimeKeys(100);
a.oneTimeKeys();                                // JSON { curve25519: { <id>: <publicKey> } }
a.markKeysAsPublished();
a.generateFallbackKey();
a.fallbackKey();
a.sign("message");                              // base64 Ed25519 sig
const restored = Account.fromPickle(a.pickle());

// Outbound:
const session: Session = a.createOutboundSession(theirIdKey, theirOTK);
const { type, body } = JSON.parse(session.encrypt("hi")); // type=0 prekey, 1 normal

// Inbound (identity key extracted from the prekey message):
const result: InboundResult = b.createInboundSession(prekeyBody);
const session2 = result.takeSession();
result.plaintext; result.senderIdentityKey;

// Persistence:
const restored2 = Session.fromPickle(session.pickle());
```

## API surface — Megolm (group)

```ts
import { GroupSession, InboundGroupSession } from "@kinsh/vodozemac-wasm";

// Sender: one outbound session per (group, sender device).
const outbound = new GroupSession();
outbound.sessionId();
outbound.sessionKey();      // share with members over Olm; decryptable from current index onward
outbound.messageIndex();    // index of the NEXT message
const body = outbound.encrypt("hello group");   // base64, no type field

// Member:
const inbound = new InboundGroupSession(sessionKeyFromSender);
inbound.firstKnownIndex();
const { plaintext, messageIndex } = JSON.parse(inbound.decrypt(body));
// Replay protection is the application's job: track (sessionId, messageIndex).

// History sharing (loses the signing chain — see vodozemac docs):
const exported = inbound.exportAt(2);           // undefined if below firstKnownIndex
const imported = InboundGroupSession.import(exported);

// Persistence:
GroupSession.fromPickle(outbound.pickle());
InboundGroupSession.fromPickle(inbound.pickle());
```

## Build

### WASM (`@kinsh/vodozemac-wasm`)

Requires Rust 1.85+ and `wasm-pack`.

```sh
npm run build       # pkg-web (browser ESM) + pkg-node (Node CJS-style)
npm test            # olm + megolm smoke tests and cross-platform vector
                    # verification against pkg-node
```

### React Native (`@kinsh/vodozemac-rn`)

Android (any OS with the NDK + `cargo-ndk`):

```sh
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
cargo ndk -P 24 -t arm64-v8a -t armeabi-v7a -t x86_64 \
  -o rn/android/src/main/jniLibs build -p vodozemac-ffi --release
cargo run -p vodozemac-ffi --bin uniffi-bindgen --release -- generate \
  --library target/aarch64-linux-android/release/libvodozemac_ffi.so \
  --language kotlin --out-dir rn/android/src/main/java
```

iOS (macOS only — see `apple/build_xcframework.sh` / the release GitHub Actions workflow):

```sh
apple/build_xcframework.sh   # rebuilds VodozemacFFI.xcframework + VodozemacFFI.swift
```

The generated `VodozemacFFI.swift` and the xcframework's static libraries must always come from the same crate build — never update one without the other.

## Attribution

- Cryptography: [vodozemac](https://github.com/matrix-org/vodozemac) (Matrix.org Foundation, Apache-2.0).
- Original Olm bindings: [dTelecom/vodozemac-wasm](https://github.com/dTelecom/vodozemac-wasm) (Apache-2.0).
- Megolm surface + `@kinsh` packaging: this fork.

# @kinsh/vodozemac-rn

React Native bindings for [vodozemac](https://github.com/matrix-org/vodozemac)
(Olm + Megolm primitives), exposed via a UniFFI-generated Swift + Kotlin
native bridge. Drop-in replacement for `@kinsh/vodozemac-wasm` on the React
Native target — same JS surface (`Account`, `Session`, `InboundResult`,
`GroupSession`, `InboundGroupSession`), same JSON-string shapes, same
pickle format.

Fork of dTelecom's `@dtelecom/vodozemac-rn` (Apache-2.0) with the Megolm
group-session surface added.

## Why a native bridge instead of WASM

WebAssembly is not yet shipped in any React Native release. Hermes V1
(RN 0.84+) has runtime Wasm support as of the Feb 2026 preview, but the
implementation is in Meta's internal monorepo and isn't reachable from
any public Hermes commit or RN-bundled prebuilt as of mid-2026. This
package routes around the engine entirely: Rust → UniFFI → Swift/Kotlin
→ TurboModule. No engine dependency, no polyfills.

## Install

```sh
npm install @kinsh/vodozemac-rn react-native-get-random-values
```

Then once at the top of your app entry (`index.js`), before importing
anything else:

```ts
import "react-native-get-random-values";
```

The package ships prebuilt artifacts (iOS XCFramework + Android .so for
arm64-v8a, armeabi-v7a, x86_64) — no Rust toolchain needed on the
consumer's machine.

## Use — Olm (1:1)

```ts
import init, { Account } from "@kinsh/vodozemac-rn";

await init();                         // no-op on RN; included for parity
                                      // with @kinsh/vodozemac-wasm

const account = Account.new();
account.generateOneTimeKeys(50);

const ids = JSON.parse(account.identityKeys());
// { curve25519: "...", ed25519: "..." }

const otks = JSON.parse(account.oneTimeKeys()).curve25519;
// { "<base64KeyId>": "<base64PublicKey>", ... }
account.markKeysAsPublished();

// Serialize for persistence — caller is responsible for at-rest encryption.
const pickle = account.pickle();
// later, restore:
const restored = Account.fromPickle(pickle);

// Release the native handle eagerly. The package also registers a
// FinalizationRegistry callback so GC will close handles eventually,
// but `close()` is the deterministic path.
account.close();
```

## Use — Megolm (group)

```ts
import { GroupSession, InboundGroupSession } from "@kinsh/vodozemac-rn";

// Sender side — one outbound session per (group, sender device):
const outbound = new GroupSession();
const keyForMembers = outbound.sessionKey();   // distribute over Olm
const body = outbound.encrypt("hello group");  // base64 megolm message

// Member side:
const inbound = new InboundGroupSession(keyForMembers);
const { plaintext, messageIndex } = JSON.parse(inbound.decrypt(body));
// Replay protection is the app's job: track (sessionId, messageIndex).

// History sharing / persistence:
const exported = inbound.exportAt(0);          // undefined below firstKnownIndex()
const imported = InboundGroupSession.import(exported!);
const restored = InboundGroupSession.fromPickle(inbound.pickle());
```

Megolm is pinned to version 1 (the Matrix-production format) on all
platforms; ciphertext and pickles interoperate with `@kinsh/vodozemac-wasm`.

## API

Identical to `@kinsh/vodozemac-wasm`. See its README for the full
method list. Method names, argument shapes, and return-value JSON
schemas are bit-for-bit compatible — SDK code doesn't know which
target it's on.

## Platforms

- iOS 15.1+ (arm64 device + arm64/x86_64 simulator)
- Android API 24+ (arm64-v8a, armeabi-v7a, x86_64)
- React Native 0.76+ (new architecture / bridgeless)

## License

Apache-2.0

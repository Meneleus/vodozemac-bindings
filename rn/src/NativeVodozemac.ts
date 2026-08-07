// TurboModule spec for the native vodozemac bridge. RN's codegen reads
// this file to generate iOS protocol (RTNVodozemacSpec) and Android
// abstract class (NativeVodozemacSpec) that the native impl conforms to.
//
// Design — opaque handles, not host objects:
//
// Each Rust handle (Account, Session) lives in a native side-table
// keyed by a `Double` (TurboModule's numeric type, JS Number under the
// hood). The JS-side classes in `index.ts` wrap the handle + dispatch
// methods through this spec. Memory is reclaimed via JS FinalizationRegistry
// callbacks that invoke `*Close()`.
//
// All methods are synchronous. vodozemac is pure computation (no I/O),
// and Hermes calls TurboModule methods on the JS thread — wrapping in
// Promise just adds latency.

import type { TurboModule } from "react-native";
import { TurboModuleRegistry } from "react-native";

/** Return shape for {@link Spec.accountCreateInboundSession}. */
export interface InboundResult {
  /** Handle to the new Session, hands ownership to the caller. */
  sessionHandle: number;
  /** Plaintext of the initial prekey message. */
  plaintext: string;
  /** Base64 curve25519 identity key extracted from the prekey message. */
  senderIdentityKey: string;
}

export interface Spec extends TurboModule {
  // ── Account ────────────────────────────────────────────────────────────

  // IMPORTANT: every method returns a value (no `void`). RN's
  // ObjCTurboModule dispatches `VoidKind` returns on a background queue
  // (fire-and-forget); subsequent reads from JS can then race ahead of
  // the mutation. Returning a sentinel forces `BooleanKind` / `NumberKind`
  // synchronous dispatch on the JS thread. The JS wrappers in `index.ts`
  // ignore the boolean.

  accountNew(): number;
  accountFromPickle(pickle: string): number;
  accountIdentityKeys(handle: number): string;
  accountGenerateOneTimeKeys(handle: number, count: number): boolean;
  accountOneTimeKeys(handle: number): string;
  accountMarkKeysAsPublished(handle: number): boolean;
  accountMaxNumberOfOneTimeKeys(handle: number): number;
  accountGenerateFallbackKey(handle: number): boolean;
  accountFallbackKey(handle: number): string;
  accountSign(handle: number, message: string): string;
  accountPickle(handle: number): string;
  accountCreateOutboundSession(
    handle: number,
    theirIdentityKey: string,
    theirOneTimeKey: string,
  ): number;
  accountCreateInboundSession(
    handle: number,
    prekeyMessageBody: string,
  ): InboundResult;
  accountClose(handle: number): boolean;

  // ── Session ────────────────────────────────────────────────────────────

  sessionFromPickle(pickle: string): number;
  sessionEncrypt(handle: number, plaintext: string): string;
  sessionDecrypt(handle: number, messageType: number, body: string): string;
  sessionSessionId(handle: number): string;
  sessionHasReceivedMessage(handle: number): boolean;
  sessionPickle(handle: number): string;
  sessionClose(handle: number): boolean;

  // ── GroupSession (outbound megolm) ─────────────────────────────────────

  groupSessionNew(): number;
  groupSessionFromPickle(pickle: string): number;
  groupSessionSessionId(handle: number): string;
  groupSessionSessionKey(handle: number): string;
  groupSessionMessageIndex(handle: number): number;
  groupSessionEncrypt(handle: number, plaintext: string): string;
  groupSessionPickle(handle: number): string;
  groupSessionClose(handle: number): boolean;

  // ── InboundGroupSession (megolm, decrypt-only) ─────────────────────────

  inboundGroupSessionNew(sessionKey: string): number;
  inboundGroupSessionImport(exportedSessionKey: string): number;
  inboundGroupSessionFromPickle(pickle: string): number;
  inboundGroupSessionSessionId(handle: number): string;
  inboundGroupSessionFirstKnownIndex(handle: number): number;
  /** Returns JSON `{ "plaintext": "...", "messageIndex": n }`. */
  inboundGroupSessionDecrypt(handle: number, message: string): string;
  /**
   * Returns the exported key (base64), or the empty string when the index
   * is below firstKnownIndex. Empty-string sentinel instead of a nullable
   * return keeps the codegen shape simple; the JS wrapper converts it to
   * `undefined`. A real exported key is never empty.
   */
  inboundGroupSessionExportAt(handle: number, index: number): string;
  inboundGroupSessionPickle(handle: number): string;
  inboundGroupSessionClose(handle: number): boolean;
}

export default TurboModuleRegistry.getEnforcing<Spec>("Vodozemac");

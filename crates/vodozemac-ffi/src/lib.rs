//! UniFFI bindings for vodozemac.
//!
//! Public surface intentionally mirrors `dtelecom-vodozemac-wasm` (the
//! sibling wasm-bindgen crate). The Olm-shaped JS contract is identical:
//! - JSON strings for structured fields (identityKeys, oneTimeKeys, encrypt
//!   result, fallbackKey) so the @dtelecom/secure-chat-client SDK can stay
//!   target-agnostic — same parsing path on web/node/RN.
//! - Pickles are JSON strings — at-rest encryption is the SDK's job.
//! - All base64 is URL-safe, no padding (vodozemac default).
//!
//! Differences from the wasm-bindgen crate:
//! - Methods that mutate `VAccount` / `VSession` take `&self` (UniFFI Object
//!   semantics, methods on `Arc<Self>`), with a `Mutex` inside for interior
//!   mutability.
//! - Errors are a `thiserror` enum exposed across the FFI boundary as
//!   `VodozemacError` rather than raw `JsValue` strings — gives Swift /
//!   Kotlin callers structured error types and lets the TurboModule layer
//!   surface specific error codes.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use serde::Serialize;
use vodozemac::{
    megolm::{
        ExportedSessionKey, GroupSession as VGroupSession, GroupSessionPickle,
        InboundGroupSession as VInboundGroupSession, InboundGroupSessionPickle, MegolmMessage,
        SessionConfig as MegolmSessionConfig, SessionKey,
    },
    olm::{
        Account as VAccount, AccountPickle, OlmMessage, PreKeyMessage, Session as VSession,
        SessionConfig, SessionPickle,
    },
    Curve25519PublicKey, KeyId,
};

uniffi::setup_scaffolding!("vodozemac");

// ── error type ──────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum VodozemacError {
    #[error("invalid base64: {reason}")]
    InvalidBase64 { reason: String },
    #[error("decryption failed: {reason}")]
    DecryptError { reason: String },
    #[error("session establishment failed: {reason}")]
    SessionError { reason: String },
    #[error("invalid pickle: {reason}")]
    InvalidPickle { reason: String },
    #[error("invalid utf-8 in plaintext: {reason}")]
    InvalidUtf8 { reason: String },
    #[error("invalid message type: {value} (expected 0 or 1)")]
    InvalidMessageType { value: u8 },
    #[error("session already taken")]
    SessionAlreadyTaken,
    #[error("internal: {reason}")]
    Internal { reason: String },
}

type Result<T> = std::result::Result<T, VodozemacError>;

fn parse_curve(b64: &str) -> Result<Curve25519PublicKey> {
    Curve25519PublicKey::from_base64(b64)
        .map_err(|e| VodozemacError::InvalidBase64 { reason: e.to_string() })
}

fn key_id_to_string(k: KeyId) -> String {
    k.to_base64()
}

// ── public-view types serialized to JSON strings ────────────────────────────
//
// These match the wasm-bindgen crate exactly. We intentionally don't expose
// them as UniFFI Records — keeping the JSON-string shape means the JS-side
// parsing logic in @dtelecom/secure-chat-client (olm-adapter.ts) doesn't
// change between web/node and RN.

#[derive(Serialize)]
struct IdentityKeysJs {
    curve25519: String,
    ed25519: String,
}

#[derive(Serialize)]
struct OneTimeKeysJs {
    /// Map of base64 key id → base64 public key.
    curve25519: BTreeMap<String, String>,
}

#[derive(Serialize)]
struct EncryptResultJs {
    /// 0 = PreKey (first message of a fresh outbound session before peer
    /// has replied), 1 = Normal (post-handshake).
    #[serde(rename = "type")]
    msg_type: u8,
    /// base64-encoded Olm message body
    body: String,
}

// ── Account ─────────────────────────────────────────────────────────────────

#[derive(uniffi::Object)]
pub struct Account {
    inner: Mutex<VAccount>,
}

#[uniffi::export]
impl Account {
    #[uniffi::constructor]
    pub fn new() -> Arc<Self> {
        Arc::new(Self { inner: Mutex::new(VAccount::new()) })
    }

    /// Restore from a JSON pickle produced by `pickle()`.
    #[uniffi::constructor(name = "from_pickle")]
    pub fn from_pickle(pickle: String) -> Result<Arc<Self>> {
        let parsed: AccountPickle = serde_json::from_str(&pickle)
            .map_err(|e| VodozemacError::InvalidPickle { reason: e.to_string() })?;
        Ok(Arc::new(Self {
            inner: Mutex::new(VAccount::from_pickle(parsed)),
        }))
    }

    /// JSON string `{ "curve25519": "<base64>", "ed25519": "<base64>" }`.
    pub fn identity_keys(&self) -> Result<String> {
        let inner = self.lock_inner()?;
        let keys = inner.identity_keys();
        let out = IdentityKeysJs {
            curve25519: keys.curve25519.to_base64(),
            ed25519: keys.ed25519.to_base64(),
        };
        serde_json::to_string(&out)
            .map_err(|e| VodozemacError::Internal { reason: e.to_string() })
    }

    pub fn generate_one_time_keys(&self, count: u32) -> Result<()> {
        let mut inner = self.lock_inner()?;
        inner.generate_one_time_keys(count as usize);
        Ok(())
    }

    /// Returns unpublished one-time keys as a JSON string:
    /// `{ "curve25519": { "<keyId>": "<publicKey>" } }`.
    /// After `mark_keys_as_published`, the inner map is empty.
    pub fn one_time_keys(&self) -> Result<String> {
        let inner = self.lock_inner()?;
        let keys = inner.one_time_keys();
        let mut out = BTreeMap::new();
        for (id, pk) in keys.iter() {
            out.insert(key_id_to_string(*id), pk.to_base64());
        }
        let wrapper = OneTimeKeysJs { curve25519: out };
        serde_json::to_string(&wrapper)
            .map_err(|e| VodozemacError::Internal { reason: e.to_string() })
    }

    pub fn mark_keys_as_published(&self) -> Result<()> {
        let mut inner = self.lock_inner()?;
        inner.mark_keys_as_published();
        Ok(())
    }

    pub fn max_number_of_one_time_keys(&self) -> Result<u32> {
        let inner = self.lock_inner()?;
        Ok(inner.max_number_of_one_time_keys() as u32)
    }

    pub fn generate_fallback_key(&self) -> Result<()> {
        let mut inner = self.lock_inner()?;
        inner.generate_fallback_key();
        Ok(())
    }

    /// Returns the unpublished fallback key as a JSON string in the same
    /// shape as `one_time_keys()` — `{ "curve25519": { "<id>": "<pub>" } }`.
    /// Empty inner map if no unpublished fallback exists.
    pub fn fallback_key(&self) -> Result<String> {
        let inner = self.lock_inner()?;
        let keys = inner.fallback_key();
        let mut out = BTreeMap::new();
        for (id, pk) in keys.iter() {
            out.insert(key_id_to_string(*id), pk.to_base64());
        }
        let wrapper = OneTimeKeysJs { curve25519: out };
        serde_json::to_string(&wrapper)
            .map_err(|e| VodozemacError::Internal { reason: e.to_string() })
    }

    /// Sign the given message with this account's Ed25519 identity key,
    /// returning a base64 signature.
    pub fn sign(&self, message: String) -> Result<String> {
        let inner = self.lock_inner()?;
        Ok(inner.sign(message.as_bytes()).to_base64())
    }

    /// Returns the JSON pickle string.
    pub fn pickle(&self) -> Result<String> {
        let inner = self.lock_inner()?;
        let p = inner.pickle();
        serde_json::to_string(&p)
            .map_err(|e| VodozemacError::Internal { reason: e.to_string() })
    }

    /// Create an outbound Olm session targeting a peer device, using their
    /// identity key + one-time-key (or fallback prekey when pool is empty).
    /// Both keys are base64.
    pub fn create_outbound_session(
        &self,
        their_identity_key: String,
        their_one_time_key: String,
    ) -> Result<Arc<Session>> {
        let inner = self.lock_inner()?;
        let id_key = parse_curve(&their_identity_key)?;
        let otk = parse_curve(&their_one_time_key)?;
        let session = inner
            .create_outbound_session(SessionConfig::version_1(), id_key, otk)
            .map_err(|e| VodozemacError::SessionError { reason: e.to_string() })?;
        Ok(Arc::new(Session { inner: Mutex::new(session) }))
    }

    /// Create an inbound session from a received prekey message body.
    /// The peer's identity key is extracted from the message itself
    /// (libolm-compatible behaviour). Returns the new session and the
    /// decrypted plaintext of the initial message together.
    pub fn create_inbound_session(
        &self,
        prekey_message_body: String,
    ) -> Result<Arc<InboundResult>> {
        let mut inner = self.lock_inner()?;
        let pkm = PreKeyMessage::from_base64(&prekey_message_body)
            .map_err(|e| VodozemacError::InvalidBase64 { reason: e.to_string() })?;
        let id_key = pkm.identity_key();
        let result = inner
            .create_inbound_session(SessionConfig::version_1(), id_key, &pkm)
            .map_err(|e| VodozemacError::SessionError { reason: e.to_string() })?;
        let plaintext = String::from_utf8(result.plaintext)
            .map_err(|e| VodozemacError::InvalidUtf8 { reason: e.to_string() })?;
        Ok(Arc::new(InboundResult {
            session: Mutex::new(Some(Arc::new(Session {
                inner: Mutex::new(result.session),
            }))),
            plaintext,
            sender_identity_key: id_key.to_base64(),
        }))
    }
}

impl Account {
    fn lock_inner(&self) -> Result<std::sync::MutexGuard<'_, VAccount>> {
        self.inner
            .lock()
            .map_err(|_| VodozemacError::Internal { reason: "account mutex poisoned".into() })
    }
}

// ── Session ─────────────────────────────────────────────────────────────────

#[derive(uniffi::Object)]
pub struct Session {
    inner: Mutex<VSession>,
}

#[uniffi::export]
impl Session {
    /// Restore from a JSON pickle produced by `pickle()`.
    #[uniffi::constructor(name = "from_pickle")]
    pub fn from_pickle(pickle: String) -> Result<Arc<Self>> {
        let parsed: SessionPickle = serde_json::from_str(&pickle)
            .map_err(|e| VodozemacError::InvalidPickle { reason: e.to_string() })?;
        Ok(Arc::new(Self {
            inner: Mutex::new(VSession::from_pickle(parsed)),
        }))
    }

    /// JSON string `{ "type": 0|1, "body": "<base64>" }`. Type 0 = PreKey,
    /// 1 = Normal.
    pub fn encrypt(&self, plaintext: String) -> Result<String> {
        let mut inner = self.lock_inner()?;
        let msg = inner
            .encrypt(plaintext.as_bytes())
            .map_err(|e| VodozemacError::Internal { reason: e.to_string() })?;
        let out = match msg {
            OlmMessage::PreKey(pkm) => EncryptResultJs {
                msg_type: 0,
                body: pkm.to_base64(),
            },
            OlmMessage::Normal(m) => EncryptResultJs {
                msg_type: 1,
                body: m.to_base64(),
            },
        };
        serde_json::to_string(&out)
            .map_err(|e| VodozemacError::Internal { reason: e.to_string() })
    }

    /// Decrypt a message of the given type (0 = PreKey, 1 = Normal).
    pub fn decrypt(&self, message_type: u8, body: String) -> Result<String> {
        let mut inner = self.lock_inner()?;
        let msg = match message_type {
            0 => OlmMessage::PreKey(
                PreKeyMessage::from_base64(&body)
                    .map_err(|e| VodozemacError::InvalidBase64 { reason: e.to_string() })?,
            ),
            1 => OlmMessage::Normal(
                vodozemac::olm::Message::from_base64(&body)
                    .map_err(|e| VodozemacError::InvalidBase64 { reason: e.to_string() })?,
            ),
            _ => return Err(VodozemacError::InvalidMessageType { value: message_type }),
        };
        let plaintext = inner
            .decrypt(&msg)
            .map_err(|e| VodozemacError::DecryptError { reason: e.to_string() })?;
        String::from_utf8(plaintext)
            .map_err(|e| VodozemacError::InvalidUtf8 { reason: e.to_string() })
    }

    pub fn session_id(&self) -> Result<String> {
        let inner = self.lock_inner()?;
        Ok(inner.session_id())
    }

    pub fn has_received_message(&self) -> Result<bool> {
        let inner = self.lock_inner()?;
        Ok(inner.has_received_message())
    }

    /// Returns the JSON pickle string.
    pub fn pickle(&self) -> Result<String> {
        let inner = self.lock_inner()?;
        let p = inner.pickle();
        serde_json::to_string(&p)
            .map_err(|e| VodozemacError::Internal { reason: e.to_string() })
    }
}

impl Session {
    fn lock_inner(&self) -> Result<std::sync::MutexGuard<'_, VSession>> {
        self.inner
            .lock()
            .map_err(|_| VodozemacError::Internal { reason: "session mutex poisoned".into() })
    }
}

// ── InboundResult ──────────────────────────────────────────────────────────

#[derive(uniffi::Object)]
pub struct InboundResult {
    // Wrapped in Mutex<Option<...>> so `take_session()` can move it out
    // exactly once across the FFI boundary.
    session: Mutex<Option<Arc<Session>>>,
    plaintext: String,
    sender_identity_key: String,
}

#[uniffi::export]
impl InboundResult {
    /// Take ownership of the new session. Errors if called twice.
    pub fn take_session(&self) -> Result<Arc<Session>> {
        let mut slot = self
            .session
            .lock()
            .map_err(|_| VodozemacError::Internal { reason: "inbound mutex poisoned".into() })?;
        slot.take().ok_or(VodozemacError::SessionAlreadyTaken)
    }

    pub fn plaintext(&self) -> String {
        self.plaintext.clone()
    }

    /// The peer's curve25519 identity key, extracted from the prekey message.
    /// Useful for on-prekey-message new-device discovery.
    pub fn sender_identity_key(&self) -> String {
        self.sender_identity_key.clone()
    }
}

// ── Megolm (group sessions) ────────────────────────────────────────────────
//
// Pinned to megolm **version 1** — the format Matrix runs in production
// (AES-256 + HMAC truncated to 8 bytes). vodozemac's version 2 (untruncated
// MAC) is gated behind the `experimental-session-config` feature and not
// production-vetted, so we don't use it. Must match the wasm crate.

fn megolm_config() -> MegolmSessionConfig {
    MegolmSessionConfig::version_1()
}

/// Decrypt result for `InboundGroupSession::decrypt()` — serialized to a
/// JSON string, same shape as the wasm crate.
#[derive(Serialize)]
struct GroupDecryptResultJs {
    plaintext: String,
    /// Ratchet index of the decrypted message. The application layer is
    /// responsible for replay protection: track (sessionId, messageIndex)
    /// pairs and reject duplicates.
    #[serde(rename = "messageIndex")]
    message_index: u32,
}

/// Outbound megolm session — one per (circle, sender device). The sender
/// encrypts with this and shares `session_key()` with members over Olm.
#[derive(uniffi::Object)]
pub struct GroupSession {
    inner: Mutex<VGroupSession>,
}

#[uniffi::export]
impl GroupSession {
    #[uniffi::constructor]
    pub fn new() -> Arc<Self> {
        Arc::new(Self { inner: Mutex::new(VGroupSession::new(megolm_config())) })
    }

    /// Restore from a JSON pickle produced by `pickle()`.
    #[uniffi::constructor(name = "from_pickle")]
    pub fn from_pickle(pickle: String) -> Result<Arc<Self>> {
        let parsed: GroupSessionPickle = serde_json::from_str(&pickle)
            .map_err(|e| VodozemacError::InvalidPickle { reason: e.to_string() })?;
        Ok(Arc::new(Self {
            inner: Mutex::new(VGroupSession::from_pickle(parsed)),
        }))
    }

    pub fn session_id(&self) -> Result<String> {
        let inner = self.lock_inner()?;
        Ok(inner.session_id())
    }

    /// Base64 session key at the **current** ratchet index. Share this with
    /// group members (over Olm); they construct an `InboundGroupSession`
    /// from it and can decrypt everything from this index onward.
    pub fn session_key(&self) -> Result<String> {
        let inner = self.lock_inner()?;
        Ok(inner.session_key().to_base64())
    }

    /// Ratchet index of the **next** message to be encrypted.
    pub fn message_index(&self) -> Result<u32> {
        let inner = self.lock_inner()?;
        Ok(inner.message_index())
    }

    /// Encrypt, returning the base64 megolm message body. No type field —
    /// megolm has a single message kind (contrast with Olm's prekey/normal).
    pub fn encrypt(&self, plaintext: String) -> Result<String> {
        let mut inner = self.lock_inner()?;
        Ok(inner.encrypt(plaintext.as_bytes()).to_base64())
    }

    /// Returns the JSON pickle string.
    pub fn pickle(&self) -> Result<String> {
        let inner = self.lock_inner()?;
        let p = inner.pickle();
        serde_json::to_string(&p)
            .map_err(|e| VodozemacError::Internal { reason: e.to_string() })
    }
}

impl GroupSession {
    fn lock_inner(&self) -> Result<std::sync::MutexGuard<'_, VGroupSession>> {
        self.inner
            .lock()
            .map_err(|_| VodozemacError::Internal { reason: "group session mutex poisoned".into() })
    }
}

/// Inbound megolm session — one per received session key. Decrypt-only.
#[derive(uniffi::Object)]
pub struct InboundGroupSession {
    inner: Mutex<VInboundGroupSession>,
}

#[uniffi::export]
impl InboundGroupSession {
    /// Construct from a base64 session key produced by
    /// `GroupSession::session_key()`.
    #[uniffi::constructor]
    pub fn new(session_key: String) -> Result<Arc<Self>> {
        let key = SessionKey::from_base64(&session_key)
            .map_err(|e| VodozemacError::InvalidBase64 { reason: e.to_string() })?;
        Ok(Arc::new(Self {
            inner: Mutex::new(VInboundGroupSession::new(&key, megolm_config())),
        }))
    }

    /// Construct from a base64 **exported** key produced by `export_at()`.
    /// Exported keys lose the signing chain, so sessions imported this way
    /// can decrypt but cannot prove who created the session.
    ///
    /// Named `import_session` (not `import`) because `import` is a hard
    /// keyword in both Kotlin and Swift.
    #[uniffi::constructor(name = "import_session")]
    pub fn import_session(exported_session_key: String) -> Result<Arc<Self>> {
        let key = ExportedSessionKey::from_base64(&exported_session_key)
            .map_err(|e| VodozemacError::InvalidBase64 { reason: e.to_string() })?;
        Ok(Arc::new(Self {
            inner: Mutex::new(VInboundGroupSession::import(&key, megolm_config())),
        }))
    }

    /// Restore from a JSON pickle produced by `pickle()`.
    #[uniffi::constructor(name = "from_pickle")]
    pub fn from_pickle(pickle: String) -> Result<Arc<Self>> {
        let parsed: InboundGroupSessionPickle = serde_json::from_str(&pickle)
            .map_err(|e| VodozemacError::InvalidPickle { reason: e.to_string() })?;
        Ok(Arc::new(Self {
            inner: Mutex::new(VInboundGroupSession::from_pickle(parsed)),
        }))
    }

    pub fn session_id(&self) -> Result<String> {
        let inner = self.lock_inner()?;
        Ok(inner.session_id())
    }

    /// Lowest ratchet index this session can decrypt.
    pub fn first_known_index(&self) -> Result<u32> {
        let inner = self.lock_inner()?;
        Ok(inner.first_known_index())
    }

    /// Decrypt a base64 megolm message body. Returns a JSON string
    /// `{ "plaintext": "...", "messageIndex": n }`.
    pub fn decrypt(&self, message: String) -> Result<String> {
        let mut inner = self.lock_inner()?;
        let msg = MegolmMessage::from_base64(&message)
            .map_err(|e| VodozemacError::InvalidBase64 { reason: e.to_string() })?;
        let decrypted = inner
            .decrypt(&msg)
            .map_err(|e| VodozemacError::DecryptError { reason: e.to_string() })?;
        let out = GroupDecryptResultJs {
            plaintext: String::from_utf8(decrypted.plaintext)
                .map_err(|e| VodozemacError::InvalidUtf8 { reason: e.to_string() })?,
            message_index: decrypted.message_index,
        };
        serde_json::to_string(&out)
            .map_err(|e| VodozemacError::Internal { reason: e.to_string() })
    }

    /// Export the session key at the given ratchet index (base64), e.g. for
    /// sharing history with a newly joined member. Returns `None` when the
    /// index is below `first_known_index()`; future indexes are computable
    /// by ratcheting forward.
    pub fn export_at(&self, index: u32) -> Result<Option<String>> {
        let mut inner = self.lock_inner()?;
        Ok(inner.export_at(index).map(|k| k.to_base64()))
    }

    /// Returns the JSON pickle string.
    pub fn pickle(&self) -> Result<String> {
        let inner = self.lock_inner()?;
        let p = inner.pickle();
        serde_json::to_string(&p)
            .map_err(|e| VodozemacError::Internal { reason: e.to_string() })
    }
}

impl InboundGroupSession {
    fn lock_inner(&self) -> Result<std::sync::MutexGuard<'_, VInboundGroupSession>> {
        self.inner.lock().map_err(|_| VodozemacError::Internal {
            reason: "inbound group session mutex poisoned".into(),
        })
    }
}

// ── tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// End-to-end roundtrip: Alice and Bob each create an Account, publish
    /// keys, Alice sends a prekey message to Bob, Bob decrypts it and
    /// replies, Alice decrypts the reply. Exercises every method needed
    /// for the first two messages of a fresh Olm session.
    #[test]
    fn prekey_and_reply_roundtrip() {
        let alice = Account::new();
        let bob = Account::new();
        bob.generate_one_time_keys(1).unwrap();

        // Pull Bob's identity + one OTK as base64 strings (the SDK gets
        // these via the key bundle wire format; here we extract from JSON).
        let bob_identity: serde_json::Value =
            serde_json::from_str(&bob.identity_keys().unwrap()).unwrap();
        let bob_id_curve = bob_identity["curve25519"].as_str().unwrap().to_string();

        let bob_otks: serde_json::Value =
            serde_json::from_str(&bob.one_time_keys().unwrap()).unwrap();
        let (_bob_otk_id, bob_otk) = bob_otks["curve25519"]
            .as_object()
            .unwrap()
            .iter()
            .next()
            .unwrap();
        let bob_otk = bob_otk.as_str().unwrap().to_string();
        bob.mark_keys_as_published().unwrap();

        // Alice → Bob: prekey message
        let alice_session = alice
            .create_outbound_session(bob_id_curve.clone(), bob_otk.clone())
            .unwrap();
        let enc_out = alice_session.encrypt("hello bob".to_string()).unwrap();
        let enc: serde_json::Value = serde_json::from_str(&enc_out).unwrap();
        assert_eq!(enc["type"], 0); // PreKey
        let alice_to_bob_body = enc["body"].as_str().unwrap().to_string();

        // Bob receives + decrypts via create_inbound_session
        let inbound = bob.create_inbound_session(alice_to_bob_body).unwrap();
        assert_eq!(inbound.plaintext(), "hello bob");
        let bob_session = inbound.take_session().unwrap();

        // Bob → Alice: reply (now Normal type)
        let reply = bob_session.encrypt("hi alice".to_string()).unwrap();
        let reply_v: serde_json::Value = serde_json::from_str(&reply).unwrap();
        // Bob hasn't been written to by Alice's session response yet, so
        // his outbound is still PreKey-shaped from libolm's perspective.
        // The important property is round-trip decoding works:
        let reply_type = reply_v["type"].as_u64().unwrap() as u8;
        let reply_body = reply_v["body"].as_str().unwrap().to_string();

        let decoded = alice_session.decrypt(reply_type, reply_body).unwrap();
        assert_eq!(decoded, "hi alice");
    }

    /// Pickle round-trip: account survives serialize → deserialize.
    #[test]
    fn account_pickle_roundtrip() {
        let acc = Account::new();
        acc.generate_one_time_keys(2).unwrap();
        let keys_before = acc.identity_keys().unwrap();
        let one_time_before = acc.one_time_keys().unwrap();

        let pickled = acc.pickle().unwrap();
        let restored = Account::from_pickle(pickled).unwrap();
        assert_eq!(restored.identity_keys().unwrap(), keys_before);
        assert_eq!(restored.one_time_keys().unwrap(), one_time_before);
    }

    /// take_session is one-shot.
    #[test]
    fn inbound_take_session_only_once() {
        // Build a minimal prekey scenario inline so we can test InboundResult.
        let alice = Account::new();
        let bob = Account::new();
        bob.generate_one_time_keys(1).unwrap();

        let bob_id_keys: serde_json::Value =
            serde_json::from_str(&bob.identity_keys().unwrap()).unwrap();
        let bob_id = bob_id_keys["curve25519"].as_str().unwrap().to_string();
        let bob_otks: serde_json::Value =
            serde_json::from_str(&bob.one_time_keys().unwrap()).unwrap();
        let bob_otk = bob_otks["curve25519"]
            .as_object()
            .unwrap()
            .values()
            .next()
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();

        let alice_sess = alice.create_outbound_session(bob_id, bob_otk).unwrap();
        let enc: serde_json::Value =
            serde_json::from_str(&alice_sess.encrypt("hi".into()).unwrap()).unwrap();
        let body = enc["body"].as_str().unwrap().to_string();

        let inbound = bob.create_inbound_session(body).unwrap();
        assert!(inbound.take_session().is_ok());
        assert!(matches!(
            inbound.take_session(),
            Err(VodozemacError::SessionAlreadyTaken)
        ));
    }

    /// Megolm: outbound → inbound round-trip, ratchet indexes, pickling,
    /// late-join semantics, and export/import.
    #[test]
    fn megolm_roundtrip_pickle_and_export() {
        let outbound = GroupSession::new();
        assert_eq!(outbound.message_index().unwrap(), 0);
        let session_key = outbound.session_key().unwrap();
        let session_id = outbound.session_id().unwrap();

        let inbound = InboundGroupSession::new(session_key).unwrap();
        assert_eq!(inbound.session_id().unwrap(), session_id);
        assert_eq!(inbound.first_known_index().unwrap(), 0);

        // Round-trip three messages; indexes advance.
        let bodies: Vec<String> = ["first", "second", "third"]
            .iter()
            .map(|m| outbound.encrypt(m.to_string()).unwrap())
            .collect();
        assert_eq!(outbound.message_index().unwrap(), 3);
        for (i, body) in bodies.iter().enumerate() {
            let out: serde_json::Value =
                serde_json::from_str(&inbound.decrypt(body.clone()).unwrap()).unwrap();
            assert_eq!(out["plaintext"], ["first", "second", "third"][i]);
            assert_eq!(out["messageIndex"], i as u64);
        }

        // Pickle round-trip on both sides; session keeps working.
        let outbound2 = GroupSession::from_pickle(outbound.pickle().unwrap()).unwrap();
        let inbound2 =
            InboundGroupSession::from_pickle(inbound.pickle().unwrap()).unwrap();
        let m4 = outbound2.encrypt("fourth".to_string()).unwrap();
        let out: serde_json::Value =
            serde_json::from_str(&inbound2.decrypt(m4).unwrap()).unwrap();
        assert_eq!(out["plaintext"], "fourth");

        // Late joiner gets the current ratchet; history is unreadable.
        let late =
            InboundGroupSession::new(outbound2.session_key().unwrap()).unwrap();
        assert_eq!(late.first_known_index().unwrap(), 4);
        assert!(late.decrypt(bodies[0].clone()).is_err());

        // Export at index 2 + import; index 0 stays unreadable there too.
        let exported = inbound2.export_at(2).unwrap().expect("export at known index");
        let imported = InboundGroupSession::import_session(exported).unwrap();
        assert_eq!(imported.first_known_index().unwrap(), 2);
        let out: serde_json::Value = serde_json::from_str(
            &imported.decrypt(bodies[2].clone()).unwrap(),
        )
        .unwrap();
        assert_eq!(out["plaintext"], "third");
        assert!(imported.decrypt(bodies[0].clone()).is_err());
        assert!(imported.export_at(0).unwrap().is_none());
    }
}

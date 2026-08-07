//! Browser WASM bindings for vodozemac.
//!
//! Exposes a small Olm-shaped surface (Account, Session, encrypt/decrypt,
//! pickle/unpickle) for non-Matrix 1:1 chat protocols. No matrix-specific
//! types, no protocol assumptions about user/device id shape.
//!
//! All binary fields cross the JS boundary as base64 (URL-safe, no padding,
//! per vodozemac's defaults). Pickles are JSON strings — at-rest encryption
//! is the application's responsibility.

use std::collections::BTreeMap;

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
use wasm_bindgen::prelude::*;

// ── helpers ─────────────────────────────────────────────────────────────────

fn js_err<E: std::fmt::Display>(e: E) -> JsValue {
    JsValue::from_str(&e.to_string())
}

fn parse_curve(b64: &str) -> Result<Curve25519PublicKey, JsValue> {
    Curve25519PublicKey::from_base64(b64).map_err(js_err)
}

fn key_id_to_string(k: KeyId) -> String {
    k.to_base64()
}

// ── Account ─────────────────────────────────────────────────────────────────

#[wasm_bindgen]
pub struct Account {
    inner: VAccount,
}

/// Public view of `Account.identity_keys()` for the JS side.
#[derive(Serialize)]
struct IdentityKeysJs {
    curve25519: String,
    ed25519: String,
}

/// Public view of `Account.one_time_keys()` for the JS side.
#[derive(Serialize)]
struct OneTimeKeysJs {
    /// Map of base64 key id → base64 public key.
    curve25519: BTreeMap<String, String>,
}

#[wasm_bindgen]
impl Account {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Account {
        Account { inner: VAccount::new() }
    }

    /// Restore from a JSON pickle produced by `pickle()`.
    #[wasm_bindgen(js_name = fromPickle)]
    pub fn from_pickle(pickle: &str) -> Result<Account, JsValue> {
        let parsed: AccountPickle =
            serde_json::from_str(pickle).map_err(js_err)?;
        Ok(Account { inner: VAccount::from_pickle(parsed) })
    }

    /// JSON string `{ "curve25519": "<base64>", "ed25519": "<base64>" }`.
    /// Returning JSON (rather than a JS Map) keeps the API drop-in for
    /// libolm-shaped consumers — caller does `JSON.parse(account.identityKeys())`.
    #[wasm_bindgen(js_name = identityKeys)]
    pub fn identity_keys(&self) -> Result<String, JsValue> {
        let keys = self.inner.identity_keys();
        let out = IdentityKeysJs {
            curve25519: keys.curve25519.to_base64(),
            ed25519: keys.ed25519.to_base64(),
        };
        serde_json::to_string(&out).map_err(js_err)
    }

    #[wasm_bindgen(js_name = generateOneTimeKeys)]
    pub fn generate_one_time_keys(&mut self, count: usize) {
        self.inner.generate_one_time_keys(count);
    }

    /// Returns unpublished one-time keys as a JSON string:
    /// `{ "curve25519": { "<keyId>": "<publicKey>" } }`.
    /// After `markKeysAsPublished`, the inner map is empty.
    #[wasm_bindgen(js_name = oneTimeKeys)]
    pub fn one_time_keys(&self) -> Result<String, JsValue> {
        let keys = self.inner.one_time_keys();
        let mut out = BTreeMap::new();
        for (id, pk) in keys.iter() {
            out.insert(key_id_to_string(*id), pk.to_base64());
        }
        let wrapper = OneTimeKeysJs { curve25519: out };
        serde_json::to_string(&wrapper).map_err(js_err)
    }

    #[wasm_bindgen(js_name = markKeysAsPublished)]
    pub fn mark_keys_as_published(&mut self) {
        self.inner.mark_keys_as_published();
    }

    #[wasm_bindgen(js_name = maxNumberOfOneTimeKeys)]
    pub fn max_number_of_one_time_keys(&self) -> usize {
        self.inner.max_number_of_one_time_keys()
    }

    #[wasm_bindgen(js_name = generateFallbackKey)]
    pub fn generate_fallback_key(&mut self) {
        self.inner.generate_fallback_key();
    }

    /// Returns the unpublished fallback key as a JSON string in the same
    /// shape as `oneTimeKeys()` — `{ "curve25519": { "<id>": "<pub>" } }`.
    /// Empty inner map if no unpublished fallback exists.
    #[wasm_bindgen(js_name = fallbackKey)]
    pub fn fallback_key(&self) -> Result<String, JsValue> {
        let keys = self.inner.fallback_key();
        let mut out = BTreeMap::new();
        for (id, pk) in keys.iter() {
            out.insert(key_id_to_string(*id), pk.to_base64());
        }
        let wrapper = OneTimeKeysJs { curve25519: out };
        serde_json::to_string(&wrapper).map_err(js_err)
    }

    /// Sign the given message with this account's Ed25519 identity key,
    /// returning a base64-encoded signature.
    pub fn sign(&self, message: &str) -> String {
        self.inner.sign(message.as_bytes()).to_base64()
    }

    /// Returns the JSON pickle string. Persist however appropriate for
    /// the runtime.
    pub fn pickle(&self) -> Result<String, JsValue> {
        let p = self.inner.pickle();
        serde_json::to_string(&p).map_err(js_err)
    }

    /// Create an outbound Olm session targeting a peer device, using their
    /// identity key + one-time-key (or fallback prekey when pool is empty).
    /// Both keys are base64.
    #[wasm_bindgen(js_name = createOutboundSession)]
    pub fn create_outbound_session(
        &self,
        their_identity_key: &str,
        their_one_time_key: &str,
    ) -> Result<Session, JsValue> {
        let id_key = parse_curve(their_identity_key)?;
        let otk = parse_curve(their_one_time_key)?;
        let session = self
            .inner
            .create_outbound_session(SessionConfig::version_1(), id_key, otk)
            .map_err(js_err)?;
        Ok(Session { inner: session })
    }

    /// Create an inbound session from a received prekey message body.
    /// The peer's identity key is extracted from the message itself
    /// (libolm-compatible behaviour). Returns both the new session and
    /// the decrypted plaintext of the initial message in one shot.
    #[wasm_bindgen(js_name = createInboundSession)]
    pub fn create_inbound_session(
        &mut self,
        prekey_message_body: &str,
    ) -> Result<InboundResult, JsValue> {
        let pkm = PreKeyMessage::from_base64(prekey_message_body).map_err(js_err)?;
        let id_key = pkm.identity_key();
        let result = self
            .inner
            .create_inbound_session(SessionConfig::version_1(), id_key, &pkm)
            .map_err(js_err)?;
        let plaintext = String::from_utf8(result.plaintext).map_err(js_err)?;
        Ok(InboundResult {
            session: Some(Session { inner: result.session }),
            plaintext,
            sender_identity_key: id_key.to_base64(),
        })
    }
}

impl Default for Account {
    fn default() -> Self {
        Self::new()
    }
}

// ── Session ─────────────────────────────────────────────────────────────────

#[wasm_bindgen]
pub struct Session {
    inner: VSession,
}

/// Encrypt result mirroring libolm's shape.
#[derive(Serialize)]
struct EncryptResultJs {
    /// 0 = PreKey (first message of a fresh outbound session, before peer
    /// has replied), 1 = Normal (post-handshake message).
    #[serde(rename = "type")]
    msg_type: u8,
    /// base64-encoded Olm message body
    body: String,
}

#[wasm_bindgen]
impl Session {
    /// Restore from a JSON pickle produced by `pickle()`.
    #[wasm_bindgen(js_name = fromPickle)]
    pub fn from_pickle(pickle: &str) -> Result<Session, JsValue> {
        let parsed: SessionPickle = serde_json::from_str(pickle).map_err(js_err)?;
        Ok(Session { inner: VSession::from_pickle(parsed) })
    }

    /// JSON string `{ "type": 0|1, "body": "<base64>" }`. Type 0 = PreKey,
    /// 1 = Normal.
    pub fn encrypt(&mut self, plaintext: &str) -> Result<String, JsValue> {
        let msg = self.inner.encrypt(plaintext.as_bytes()).map_err(js_err)?;
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
        serde_json::to_string(&out).map_err(js_err)
    }

    /// Decrypt a message of the given type (0 = PreKey, 1 = Normal).
    pub fn decrypt(&mut self, message_type: u8, body: &str) -> Result<String, JsValue> {
        let msg = match message_type {
            0 => OlmMessage::PreKey(PreKeyMessage::from_base64(body).map_err(js_err)?),
            1 => OlmMessage::Normal(
                vodozemac::olm::Message::from_base64(body).map_err(js_err)?,
            ),
            _ => return Err(JsValue::from_str("unknown messageType (expected 0 or 1)")),
        };
        let plaintext = self.inner.decrypt(&msg).map_err(js_err)?;
        String::from_utf8(plaintext).map_err(js_err)
    }

    #[wasm_bindgen(js_name = sessionId)]
    pub fn session_id(&self) -> String {
        self.inner.session_id()
    }

    #[wasm_bindgen(js_name = hasReceivedMessage)]
    pub fn has_received_message(&self) -> bool {
        self.inner.has_received_message()
    }

    /// Returns the JSON pickle string.
    pub fn pickle(&self) -> Result<String, JsValue> {
        let p = self.inner.pickle();
        serde_json::to_string(&p).map_err(js_err)
    }
}

// ── InboundResult ──────────────────────────────────────────────────────────

#[wasm_bindgen]
pub struct InboundResult {
    session: Option<Session>,
    plaintext: String,
    sender_identity_key: String,
}

#[wasm_bindgen]
impl InboundResult {
    /// Take ownership of the new session. Throws if called twice.
    #[wasm_bindgen(js_name = takeSession)]
    pub fn take_session(&mut self) -> Result<Session, JsValue> {
        self.session
            .take()
            .ok_or_else(|| JsValue::from_str("session already taken"))
    }

    #[wasm_bindgen(getter)]
    pub fn plaintext(&self) -> String {
        self.plaintext.clone()
    }

    /// The peer's curve25519 identity key, extracted from the prekey
    /// message body. Useful for on-prekey-message new-device discovery —
    /// the SDK can use this to validate the sender against the cached
    /// device list (or note a new device).
    #[wasm_bindgen(getter, js_name = senderIdentityKey)]
    pub fn sender_identity_key(&self) -> String {
        self.sender_identity_key.clone()
    }
}

// ── Megolm (group sessions) ────────────────────────────────────────────────
//
// Pinned to megolm **version 1** — the format Matrix runs in production
// (AES-256 + HMAC truncated to 8 bytes). vodozemac's version 2 (untruncated
// MAC) is gated behind the `experimental-session-config` feature and not
// production-vetted, so we don't use it. All platforms (wasm + RN) must use
// the same version.

fn megolm_config() -> MegolmSessionConfig {
    MegolmSessionConfig::version_1()
}

/// Decrypt result for `InboundGroupSession.decrypt()`.
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
/// encrypts with this and shares `sessionKey()` with members over Olm.
#[wasm_bindgen]
pub struct GroupSession {
    inner: VGroupSession,
}

#[wasm_bindgen]
impl GroupSession {
    #[wasm_bindgen(constructor)]
    pub fn new() -> GroupSession {
        GroupSession { inner: VGroupSession::new(megolm_config()) }
    }

    /// Restore from a JSON pickle produced by `pickle()`.
    #[wasm_bindgen(js_name = fromPickle)]
    pub fn from_pickle(pickle: &str) -> Result<GroupSession, JsValue> {
        let parsed: GroupSessionPickle = serde_json::from_str(pickle).map_err(js_err)?;
        Ok(GroupSession { inner: VGroupSession::from_pickle(parsed) })
    }

    #[wasm_bindgen(js_name = sessionId)]
    pub fn session_id(&self) -> String {
        self.inner.session_id()
    }

    /// Base64 session key at the **current** ratchet index. Share this with
    /// group members (over Olm); they construct an `InboundGroupSession`
    /// from it and can decrypt everything from this index onward.
    #[wasm_bindgen(js_name = sessionKey)]
    pub fn session_key(&self) -> String {
        self.inner.session_key().to_base64()
    }

    /// Ratchet index of the **next** message to be encrypted.
    #[wasm_bindgen(js_name = messageIndex)]
    pub fn message_index(&self) -> u32 {
        self.inner.message_index()
    }

    /// Encrypt, returning the base64 megolm message body. No type field —
    /// megolm has a single message kind (contrast with Olm's prekey/normal).
    pub fn encrypt(&mut self, plaintext: &str) -> String {
        self.inner.encrypt(plaintext.as_bytes()).to_base64()
    }

    /// Returns the JSON pickle string.
    pub fn pickle(&self) -> Result<String, JsValue> {
        let p = self.inner.pickle();
        serde_json::to_string(&p).map_err(js_err)
    }
}

impl Default for GroupSession {
    fn default() -> Self {
        Self::new()
    }
}

/// Inbound megolm session — one per received session key. Decrypt-only.
#[wasm_bindgen]
pub struct InboundGroupSession {
    inner: VInboundGroupSession,
}

#[wasm_bindgen]
impl InboundGroupSession {
    /// Construct from a base64 session key produced by
    /// `GroupSession.sessionKey()`.
    #[wasm_bindgen(constructor)]
    pub fn new(session_key: &str) -> Result<InboundGroupSession, JsValue> {
        let key = SessionKey::from_base64(session_key).map_err(js_err)?;
        Ok(InboundGroupSession {
            inner: VInboundGroupSession::new(&key, megolm_config()),
        })
    }

    /// Construct from a base64 **exported** key produced by `exportAt()`.
    /// Exported keys lose the signing chain, so sessions imported this way
    /// can decrypt but cannot prove who created the session.
    #[wasm_bindgen(js_name = import)]
    pub fn import(exported_session_key: &str) -> Result<InboundGroupSession, JsValue> {
        let key = ExportedSessionKey::from_base64(exported_session_key).map_err(js_err)?;
        Ok(InboundGroupSession {
            inner: VInboundGroupSession::import(&key, megolm_config()),
        })
    }

    /// Restore from a JSON pickle produced by `pickle()`.
    #[wasm_bindgen(js_name = fromPickle)]
    pub fn from_pickle(pickle: &str) -> Result<InboundGroupSession, JsValue> {
        let parsed: InboundGroupSessionPickle =
            serde_json::from_str(pickle).map_err(js_err)?;
        Ok(InboundGroupSession { inner: VInboundGroupSession::from_pickle(parsed) })
    }

    #[wasm_bindgen(js_name = sessionId)]
    pub fn session_id(&self) -> String {
        self.inner.session_id()
    }

    /// Lowest ratchet index this session can decrypt.
    #[wasm_bindgen(js_name = firstKnownIndex)]
    pub fn first_known_index(&self) -> u32 {
        self.inner.first_known_index()
    }

    /// Decrypt a base64 megolm message body. Returns a JSON string
    /// `{ "plaintext": "...", "messageIndex": n }`.
    pub fn decrypt(&mut self, message: &str) -> Result<String, JsValue> {
        let msg = MegolmMessage::from_base64(message).map_err(js_err)?;
        let decrypted = self.inner.decrypt(&msg).map_err(js_err)?;
        let out = GroupDecryptResultJs {
            plaintext: String::from_utf8(decrypted.plaintext).map_err(js_err)?,
            message_index: decrypted.message_index,
        };
        serde_json::to_string(&out).map_err(js_err)
    }

    /// Export the session key at the given ratchet index (base64), e.g. for
    /// sharing history with a newly joined member. Returns `undefined` when
    /// the index is below `firstKnownIndex()`.
    #[wasm_bindgen(js_name = exportAt)]
    pub fn export_at(&mut self, index: u32) -> Option<String> {
        self.inner.export_at(index).map(|k| k.to_base64())
    }

    /// Returns the JSON pickle string.
    pub fn pickle(&self) -> Result<String, JsValue> {
        let p = self.inner.pickle();
        serde_json::to_string(&p).map_err(js_err)
    }
}

// ── module init / panic hook ───────────────────────────────────────────────

#[wasm_bindgen(start)]
pub fn module_start() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

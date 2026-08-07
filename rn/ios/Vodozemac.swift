// Swift implementation of the Vodozemac TurboModule.
//
// Holds a side-table mapping numeric handles (the JS-facing type) to
// UniFFI's `Account` / `Session` instances. The `.mm` wrapper conforms
// to the codegen-generated `NativeVodozemacSpec` and delegates every
// method to this class.
//
// Why Swift + an .mm wrapper rather than pure Obj-C++:
//   UniFFI emits plain Swift classes (not @objc-bridged), so calling
//   them from Obj-C++ directly isn't possible. The Swift side keeps the
//   UniFFI handles; the .mm side is a thin codegen-protocol facade.

import Foundation
// `Account`, `Session`, `InboundResult` come from the UniFFI-generated
// VodozemacFFI.swift, which CocoaPods compiles into the same module as
// this file. No import needed. The xcframework supplies the underlying
// `VodozemacFFIFFI` C module via the podspec's `vendored_frameworks`.

/// Errors mapped to NSError so the .mm bridge can re-throw them as
/// TurboModule errors that surface in JS as Error objects.
@objc(VodozemacErrorBridge)
public class VodozemacErrorBridge: NSObject {
    @objc public static let domain = "VodozemacError"
}

@objc(VodozemacImpl)
public class VodozemacImpl: NSObject {
    // ── side-table ─────────────────────────────────────────────────────
    // Handles are int64 to match JS Number's safe integer range. NSLock
    // protects against the (rare-but-possible) cross-thread access if a
    // consumer wraps these calls in workers or background queues.

    private static let lock = NSLock()
    private static var nextHandle: Int64 = 1
    private static var accounts: [Int64: Account] = [:]
    private static var sessions: [Int64: Session] = [:]
    private static var groupSessions: [Int64: GroupSession] = [:]
    private static var inboundGroupSessions: [Int64: InboundGroupSession] = [:]

    private static func allocHandle() -> Int64 {
        lock.lock()
        defer { lock.unlock() }
        let h = nextHandle
        nextHandle += 1
        return h
    }

    private static func registerAccount(_ a: Account) -> Int64 {
        let h = allocHandle()
        lock.lock(); defer { lock.unlock() }
        accounts[h] = a
        return h
    }

    private static func registerSession(_ s: Session) -> Int64 {
        let h = allocHandle()
        lock.lock(); defer { lock.unlock() }
        sessions[h] = s
        return h
    }

    private static func getAccount(_ h: Int64) throws -> Account {
        lock.lock(); defer { lock.unlock() }
        guard let a = accounts[h] else {
            throw NSError(
                domain: VodozemacErrorBridge.domain,
                code: 1,
                userInfo: [NSLocalizedDescriptionKey: "no account for handle \(h)"]
            )
        }
        return a
    }

    private static func getSession(_ h: Int64) throws -> Session {
        lock.lock(); defer { lock.unlock() }
        guard let s = sessions[h] else {
            throw NSError(
                domain: VodozemacErrorBridge.domain,
                code: 2,
                userInfo: [NSLocalizedDescriptionKey: "no session for handle \(h)"]
            )
        }
        return s
    }

    private static func registerGroupSession(_ s: GroupSession) -> Int64 {
        let h = allocHandle()
        lock.lock(); defer { lock.unlock() }
        groupSessions[h] = s
        return h
    }

    private static func registerInboundGroupSession(_ s: InboundGroupSession) -> Int64 {
        let h = allocHandle()
        lock.lock(); defer { lock.unlock() }
        inboundGroupSessions[h] = s
        return h
    }

    private static func getGroupSession(_ h: Int64) throws -> GroupSession {
        lock.lock(); defer { lock.unlock() }
        guard let s = groupSessions[h] else {
            throw NSError(
                domain: VodozemacErrorBridge.domain,
                code: 3,
                userInfo: [NSLocalizedDescriptionKey: "no group session for handle \(h)"]
            )
        }
        return s
    }

    private static func getInboundGroupSession(_ h: Int64) throws -> InboundGroupSession {
        lock.lock(); defer { lock.unlock() }
        guard let s = inboundGroupSessions[h] else {
            throw NSError(
                domain: VodozemacErrorBridge.domain,
                code: 4,
                userInfo: [NSLocalizedDescriptionKey: "no inbound group session for handle \(h)"]
            )
        }
        return s
    }

    // ── Account methods ────────────────────────────────────────────────

    @objc public static func accountNew() -> NSNumber {
        return NSNumber(value: registerAccount(Account()))
    }

    @objc public static func accountFromPickle(_ pickle: String) throws -> NSNumber {
        let acc = try Account.fromPickle(pickle: pickle)
        return NSNumber(value: registerAccount(acc))
    }

    @objc public static func accountIdentityKeys(_ handle: NSNumber) throws -> String {
        return try getAccount(handle.int64Value).identityKeys()
    }

    @objc public static func accountGenerateOneTimeKeys(_ handle: NSNumber, count: NSNumber) throws -> NSNumber {
        try getAccount(handle.int64Value).generateOneTimeKeys(count: count.uint32Value)
        return NSNumber(value: true)
    }

    @objc public static func accountOneTimeKeys(_ handle: NSNumber) throws -> String {
        return try getAccount(handle.int64Value).oneTimeKeys()
    }

    @objc public static func accountMarkKeysAsPublished(_ handle: NSNumber) throws -> NSNumber {
        try getAccount(handle.int64Value).markKeysAsPublished()
        return NSNumber(value: true)
    }

    @objc public static func accountMaxNumberOfOneTimeKeys(_ handle: NSNumber) throws -> NSNumber {
        return NSNumber(value: try getAccount(handle.int64Value).maxNumberOfOneTimeKeys())
    }

    @objc public static func accountGenerateFallbackKey(_ handle: NSNumber) throws -> NSNumber {
        try getAccount(handle.int64Value).generateFallbackKey()
        return NSNumber(value: true)
    }

    @objc public static func accountFallbackKey(_ handle: NSNumber) throws -> String {
        return try getAccount(handle.int64Value).fallbackKey()
    }

    @objc public static func accountSign(_ handle: NSNumber, message: String) throws -> String {
        return try getAccount(handle.int64Value).sign(message: message)
    }

    @objc public static func accountPickle(_ handle: NSNumber) throws -> String {
        return try getAccount(handle.int64Value).pickle()
    }

    @objc public static func accountCreateOutboundSession(
        _ handle: NSNumber,
        theirIdentityKey: String,
        theirOneTimeKey: String
    ) throws -> NSNumber {
        let session = try getAccount(handle.int64Value).createOutboundSession(
            theirIdentityKey: theirIdentityKey,
            theirOneTimeKey: theirOneTimeKey
        )
        return NSNumber(value: registerSession(session))
    }

    /// Returns NSDictionary `{ sessionHandle: Int64, plaintext: String, senderIdentityKey: String }`.
    /// The .mm bridge marshals NSDictionary → Obj-C++ map → JS object.
    @objc public static func accountCreateInboundSession(
        _ handle: NSNumber,
        prekeyMessageBody: String
    ) throws -> NSDictionary {
        let inbound = try getAccount(handle.int64Value).createInboundSession(prekeyMessageBody: prekeyMessageBody)
        let sessionHandle = registerSession(try inbound.takeSession())
        return [
            "sessionHandle": NSNumber(value: sessionHandle),
            "plaintext": inbound.plaintext(),
            "senderIdentityKey": inbound.senderIdentityKey(),
        ]
    }

    @objc public static func accountClose(_ handle: NSNumber) -> NSNumber {
        lock.lock(); defer { lock.unlock() }
        accounts.removeValue(forKey: handle.int64Value)
        return NSNumber(value: true)
    }

    // ── Session methods ────────────────────────────────────────────────

    @objc public static func sessionFromPickle(_ pickle: String) throws -> NSNumber {
        let s = try Session.fromPickle(pickle: pickle)
        return NSNumber(value: registerSession(s))
    }

    @objc public static func sessionEncrypt(_ handle: NSNumber, plaintext: String) throws -> String {
        return try getSession(handle.int64Value).encrypt(plaintext: plaintext)
    }

    @objc public static func sessionDecrypt(_ handle: NSNumber, messageType: NSNumber, body: String) throws -> String {
        return try getSession(handle.int64Value).decrypt(messageType: messageType.uint8Value, body: body)
    }

    @objc public static func sessionSessionId(_ handle: NSNumber) throws -> String {
        return try getSession(handle.int64Value).sessionId()
    }

    @objc public static func sessionHasReceivedMessage(_ handle: NSNumber) throws -> NSNumber {
        return NSNumber(value: try getSession(handle.int64Value).hasReceivedMessage())
    }

    @objc public static func sessionPickle(_ handle: NSNumber) throws -> String {
        return try getSession(handle.int64Value).pickle()
    }

    @objc public static func sessionClose(_ handle: NSNumber) -> NSNumber {
        lock.lock(); defer { lock.unlock() }
        sessions.removeValue(forKey: handle.int64Value)
        return NSNumber(value: true)
    }

    // ── GroupSession methods (outbound megolm) ─────────────────────────

    @objc public static func groupSessionNew() -> NSNumber {
        return NSNumber(value: registerGroupSession(GroupSession()))
    }

    @objc public static func groupSessionFromPickle(_ pickle: String) throws -> NSNumber {
        let s = try GroupSession.fromPickle(pickle: pickle)
        return NSNumber(value: registerGroupSession(s))
    }

    @objc public static func groupSessionSessionId(_ handle: NSNumber) throws -> String {
        return try getGroupSession(handle.int64Value).sessionId()
    }

    @objc public static func groupSessionSessionKey(_ handle: NSNumber) throws -> String {
        return try getGroupSession(handle.int64Value).sessionKey()
    }

    @objc public static func groupSessionMessageIndex(_ handle: NSNumber) throws -> NSNumber {
        return NSNumber(value: try getGroupSession(handle.int64Value).messageIndex())
    }

    @objc public static func groupSessionEncrypt(_ handle: NSNumber, plaintext: String) throws -> String {
        return try getGroupSession(handle.int64Value).encrypt(plaintext: plaintext)
    }

    @objc public static func groupSessionPickle(_ handle: NSNumber) throws -> String {
        return try getGroupSession(handle.int64Value).pickle()
    }

    @objc public static func groupSessionClose(_ handle: NSNumber) -> NSNumber {
        lock.lock(); defer { lock.unlock() }
        groupSessions.removeValue(forKey: handle.int64Value)
        return NSNumber(value: true)
    }

    // ── InboundGroupSession methods (megolm, decrypt-only) ─────────────

    @objc public static func inboundGroupSessionNew(_ sessionKey: String) throws -> NSNumber {
        let s = try InboundGroupSession(sessionKey: sessionKey)
        return NSNumber(value: registerInboundGroupSession(s))
    }

    @objc public static func inboundGroupSessionImport(_ exportedSessionKey: String) throws -> NSNumber {
        let s = try InboundGroupSession.importSession(exportedSessionKey: exportedSessionKey)
        return NSNumber(value: registerInboundGroupSession(s))
    }

    @objc public static func inboundGroupSessionFromPickle(_ pickle: String) throws -> NSNumber {
        let s = try InboundGroupSession.fromPickle(pickle: pickle)
        return NSNumber(value: registerInboundGroupSession(s))
    }

    @objc public static func inboundGroupSessionSessionId(_ handle: NSNumber) throws -> String {
        return try getInboundGroupSession(handle.int64Value).sessionId()
    }

    @objc public static func inboundGroupSessionFirstKnownIndex(_ handle: NSNumber) throws -> NSNumber {
        return NSNumber(value: try getInboundGroupSession(handle.int64Value).firstKnownIndex())
    }

    @objc public static func inboundGroupSessionDecrypt(_ handle: NSNumber, message: String) throws -> String {
        return try getInboundGroupSession(handle.int64Value).decrypt(message: message)
    }

    /// Empty string = "not exportable at this index" (see the TS spec).
    @objc public static func inboundGroupSessionExportAt(_ handle: NSNumber, index: NSNumber) throws -> String {
        return try getInboundGroupSession(handle.int64Value).exportAt(index: index.uint32Value) ?? ""
    }

    @objc public static func inboundGroupSessionPickle(_ handle: NSNumber) throws -> String {
        return try getInboundGroupSession(handle.int64Value).pickle()
    }

    @objc public static func inboundGroupSessionClose(_ handle: NSNumber) -> NSNumber {
        lock.lock(); defer { lock.unlock() }
        inboundGroupSessions.removeValue(forKey: handle.int64Value)
        return NSNumber(value: true)
    }
}

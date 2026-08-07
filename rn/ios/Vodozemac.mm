// TurboModule entry point for Vodozemac.
//
// Conforms to NativeVodozemacSpec (codegen-generated from
// src/NativeVodozemac.ts). Every method delegates to the Swift impl
// (VodozemacImpl in Vodozemac.swift) via the auto-generated
// Vodozemac-Swift.h header.
//
// All methods are synchronous: vodozemac is pure computation, Hermes
// already runs TurboModule calls on the JS thread, and Promise wrapping
// would just add cost.
//
// The @interface is declared inline here (no separate .h) because the
// spec import below pulls in Fabric/ReactCommon C++ headers, and we
// don't want those headers leaking into the pod's umbrella module
// where Swift would try to parse them as Objective-C.

#import <Foundation/Foundation.h>
#import <RTNVodozemacSpec/RTNVodozemacSpec.h>
// Pod can be built as a framework (CocoaPods `use_frameworks!`) — common
// in any RN app that has Swift deps — or as a static library (CocoaPods
// default). The Xcode-generated Swift interop header lives at
//   <Vodozemac/Vodozemac-Swift.h>   when built as a framework
//   "Vodozemac-Swift.h"             when built as a static lib
// Support both so we work regardless of the consumer's linkage choice.
#if __has_include(<Vodozemac/Vodozemac-Swift.h>)
#import <Vodozemac/Vodozemac-Swift.h>
#else
#import "Vodozemac-Swift.h"
#endif

@interface Vodozemac : NSObject <NativeVodozemacSpec>
@end

@implementation Vodozemac

RCT_EXPORT_MODULE()

#pragma mark - Account

- (NSNumber *)accountNew {
    return [VodozemacImpl accountNew];
}

- (NSNumber *)accountFromPickle:(NSString *)pickle {
    NSError *err = nil;
    NSNumber *r = [VodozemacImpl accountFromPickle:pickle error:&err];
    if (err) { @throw [NSException exceptionWithName:@"Vodozemac" reason:err.localizedDescription userInfo:nil]; }
    return r;
}

- (NSString *)accountIdentityKeys:(double)handle {
    NSError *err = nil;
    NSString *r = [VodozemacImpl accountIdentityKeys:@(handle) error:&err];
    if (err) { @throw [NSException exceptionWithName:@"Vodozemac" reason:err.localizedDescription userInfo:nil]; }
    return r;
}

- (NSNumber *)accountGenerateOneTimeKeys:(double)handle count:(double)count {
    NSError *err = nil;
    NSNumber *r = [VodozemacImpl accountGenerateOneTimeKeys:@(handle) count:@(count) error:&err];
    if (err) { @throw [NSException exceptionWithName:@"Vodozemac" reason:err.localizedDescription userInfo:nil]; }
    return r;
}

- (NSString *)accountOneTimeKeys:(double)handle {
    NSError *err = nil;
    NSString *r = [VodozemacImpl accountOneTimeKeys:@(handle) error:&err];
    if (err) { @throw [NSException exceptionWithName:@"Vodozemac" reason:err.localizedDescription userInfo:nil]; }
    return r;
}

- (NSNumber *)accountMarkKeysAsPublished:(double)handle {
    NSError *err = nil;
    NSNumber *r = [VodozemacImpl accountMarkKeysAsPublished:@(handle) error:&err];
    if (err) { @throw [NSException exceptionWithName:@"Vodozemac" reason:err.localizedDescription userInfo:nil]; }
    return r;
}

- (NSNumber *)accountMaxNumberOfOneTimeKeys:(double)handle {
    NSError *err = nil;
    NSNumber *r = [VodozemacImpl accountMaxNumberOfOneTimeKeys:@(handle) error:&err];
    if (err) { @throw [NSException exceptionWithName:@"Vodozemac" reason:err.localizedDescription userInfo:nil]; }
    return r;
}

- (NSNumber *)accountGenerateFallbackKey:(double)handle {
    NSError *err = nil;
    NSNumber *r = [VodozemacImpl accountGenerateFallbackKey:@(handle) error:&err];
    if (err) { @throw [NSException exceptionWithName:@"Vodozemac" reason:err.localizedDescription userInfo:nil]; }
    return r;
}

- (NSString *)accountFallbackKey:(double)handle {
    NSError *err = nil;
    NSString *r = [VodozemacImpl accountFallbackKey:@(handle) error:&err];
    if (err) { @throw [NSException exceptionWithName:@"Vodozemac" reason:err.localizedDescription userInfo:nil]; }
    return r;
}

- (NSString *)accountSign:(double)handle message:(NSString *)message {
    NSError *err = nil;
    NSString *r = [VodozemacImpl accountSign:@(handle) message:message error:&err];
    if (err) { @throw [NSException exceptionWithName:@"Vodozemac" reason:err.localizedDescription userInfo:nil]; }
    return r;
}

- (NSString *)accountPickle:(double)handle {
    NSError *err = nil;
    NSString *r = [VodozemacImpl accountPickle:@(handle) error:&err];
    if (err) { @throw [NSException exceptionWithName:@"Vodozemac" reason:err.localizedDescription userInfo:nil]; }
    return r;
}

- (NSNumber *)accountCreateOutboundSession:(double)handle
                          theirIdentityKey:(NSString *)theirIdentityKey
                           theirOneTimeKey:(NSString *)theirOneTimeKey {
    NSError *err = nil;
    NSNumber *r = [VodozemacImpl accountCreateOutboundSession:@(handle)
                                              theirIdentityKey:theirIdentityKey
                                               theirOneTimeKey:theirOneTimeKey
                                                         error:&err];
    if (err) { @throw [NSException exceptionWithName:@"Vodozemac" reason:err.localizedDescription userInfo:nil]; }
    return r;
}

- (NSDictionary *)accountCreateInboundSession:(double)handle
                            prekeyMessageBody:(NSString *)prekeyMessageBody {
    NSError *err = nil;
    NSDictionary *r = [VodozemacImpl accountCreateInboundSession:@(handle)
                                               prekeyMessageBody:prekeyMessageBody
                                                           error:&err];
    if (err) { @throw [NSException exceptionWithName:@"Vodozemac" reason:err.localizedDescription userInfo:nil]; }
    return r;
}

- (NSNumber *)accountClose:(double)handle {
    return [VodozemacImpl accountClose:@(handle)];
}

#pragma mark - Session

- (NSNumber *)sessionFromPickle:(NSString *)pickle {
    NSError *err = nil;
    NSNumber *r = [VodozemacImpl sessionFromPickle:pickle error:&err];
    if (err) { @throw [NSException exceptionWithName:@"Vodozemac" reason:err.localizedDescription userInfo:nil]; }
    return r;
}

- (NSString *)sessionEncrypt:(double)handle plaintext:(NSString *)plaintext {
    NSError *err = nil;
    NSString *r = [VodozemacImpl sessionEncrypt:@(handle) plaintext:plaintext error:&err];
    if (err) { @throw [NSException exceptionWithName:@"Vodozemac" reason:err.localizedDescription userInfo:nil]; }
    return r;
}

- (NSString *)sessionDecrypt:(double)handle messageType:(double)messageType body:(NSString *)body {
    NSError *err = nil;
    NSString *r = [VodozemacImpl sessionDecrypt:@(handle) messageType:@(messageType) body:body error:&err];
    if (err) { @throw [NSException exceptionWithName:@"Vodozemac" reason:err.localizedDescription userInfo:nil]; }
    return r;
}

- (NSString *)sessionSessionId:(double)handle {
    NSError *err = nil;
    NSString *r = [VodozemacImpl sessionSessionId:@(handle) error:&err];
    if (err) { @throw [NSException exceptionWithName:@"Vodozemac" reason:err.localizedDescription userInfo:nil]; }
    return r;
}

- (NSNumber *)sessionHasReceivedMessage:(double)handle {
    NSError *err = nil;
    NSNumber *r = [VodozemacImpl sessionHasReceivedMessage:@(handle) error:&err];
    if (err) { @throw [NSException exceptionWithName:@"Vodozemac" reason:err.localizedDescription userInfo:nil]; }
    return r;
}

- (NSString *)sessionPickle:(double)handle {
    NSError *err = nil;
    NSString *r = [VodozemacImpl sessionPickle:@(handle) error:&err];
    if (err) { @throw [NSException exceptionWithName:@"Vodozemac" reason:err.localizedDescription userInfo:nil]; }
    return r;
}

- (NSNumber *)sessionClose:(double)handle {
    return [VodozemacImpl sessionClose:@(handle)];
}

#pragma mark - GroupSession

- (NSNumber *)groupSessionNew {
    return [VodozemacImpl groupSessionNew];
}

- (NSNumber *)groupSessionFromPickle:(NSString *)pickle {
    NSError *err = nil;
    NSNumber *r = [VodozemacImpl groupSessionFromPickle:pickle error:&err];
    if (err) { @throw [NSException exceptionWithName:@"Vodozemac" reason:err.localizedDescription userInfo:nil]; }
    return r;
}

- (NSString *)groupSessionSessionId:(double)handle {
    NSError *err = nil;
    NSString *r = [VodozemacImpl groupSessionSessionId:@(handle) error:&err];
    if (err) { @throw [NSException exceptionWithName:@"Vodozemac" reason:err.localizedDescription userInfo:nil]; }
    return r;
}

- (NSString *)groupSessionSessionKey:(double)handle {
    NSError *err = nil;
    NSString *r = [VodozemacImpl groupSessionSessionKey:@(handle) error:&err];
    if (err) { @throw [NSException exceptionWithName:@"Vodozemac" reason:err.localizedDescription userInfo:nil]; }
    return r;
}

- (NSNumber *)groupSessionMessageIndex:(double)handle {
    NSError *err = nil;
    NSNumber *r = [VodozemacImpl groupSessionMessageIndex:@(handle) error:&err];
    if (err) { @throw [NSException exceptionWithName:@"Vodozemac" reason:err.localizedDescription userInfo:nil]; }
    return r;
}

- (NSString *)groupSessionEncrypt:(double)handle plaintext:(NSString *)plaintext {
    NSError *err = nil;
    NSString *r = [VodozemacImpl groupSessionEncrypt:@(handle) plaintext:plaintext error:&err];
    if (err) { @throw [NSException exceptionWithName:@"Vodozemac" reason:err.localizedDescription userInfo:nil]; }
    return r;
}

- (NSString *)groupSessionPickle:(double)handle {
    NSError *err = nil;
    NSString *r = [VodozemacImpl groupSessionPickle:@(handle) error:&err];
    if (err) { @throw [NSException exceptionWithName:@"Vodozemac" reason:err.localizedDescription userInfo:nil]; }
    return r;
}

- (NSNumber *)groupSessionClose:(double)handle {
    return [VodozemacImpl groupSessionClose:@(handle)];
}

#pragma mark - InboundGroupSession

- (NSNumber *)inboundGroupSessionNew:(NSString *)sessionKey {
    NSError *err = nil;
    NSNumber *r = [VodozemacImpl inboundGroupSessionNew:sessionKey error:&err];
    if (err) { @throw [NSException exceptionWithName:@"Vodozemac" reason:err.localizedDescription userInfo:nil]; }
    return r;
}

- (NSNumber *)inboundGroupSessionImport:(NSString *)exportedSessionKey {
    NSError *err = nil;
    NSNumber *r = [VodozemacImpl inboundGroupSessionImport:exportedSessionKey error:&err];
    if (err) { @throw [NSException exceptionWithName:@"Vodozemac" reason:err.localizedDescription userInfo:nil]; }
    return r;
}

- (NSNumber *)inboundGroupSessionFromPickle:(NSString *)pickle {
    NSError *err = nil;
    NSNumber *r = [VodozemacImpl inboundGroupSessionFromPickle:pickle error:&err];
    if (err) { @throw [NSException exceptionWithName:@"Vodozemac" reason:err.localizedDescription userInfo:nil]; }
    return r;
}

- (NSString *)inboundGroupSessionSessionId:(double)handle {
    NSError *err = nil;
    NSString *r = [VodozemacImpl inboundGroupSessionSessionId:@(handle) error:&err];
    if (err) { @throw [NSException exceptionWithName:@"Vodozemac" reason:err.localizedDescription userInfo:nil]; }
    return r;
}

- (NSNumber *)inboundGroupSessionFirstKnownIndex:(double)handle {
    NSError *err = nil;
    NSNumber *r = [VodozemacImpl inboundGroupSessionFirstKnownIndex:@(handle) error:&err];
    if (err) { @throw [NSException exceptionWithName:@"Vodozemac" reason:err.localizedDescription userInfo:nil]; }
    return r;
}

- (NSString *)inboundGroupSessionDecrypt:(double)handle message:(NSString *)message {
    NSError *err = nil;
    NSString *r = [VodozemacImpl inboundGroupSessionDecrypt:@(handle) message:message error:&err];
    if (err) { @throw [NSException exceptionWithName:@"Vodozemac" reason:err.localizedDescription userInfo:nil]; }
    return r;
}

- (NSString *)inboundGroupSessionExportAt:(double)handle index:(double)index {
    NSError *err = nil;
    NSString *r = [VodozemacImpl inboundGroupSessionExportAt:@(handle) index:@(index) error:&err];
    if (err) { @throw [NSException exceptionWithName:@"Vodozemac" reason:err.localizedDescription userInfo:nil]; }
    return r;
}

- (NSString *)inboundGroupSessionPickle:(double)handle {
    NSError *err = nil;
    NSString *r = [VodozemacImpl inboundGroupSessionPickle:@(handle) error:&err];
    if (err) { @throw [NSException exceptionWithName:@"Vodozemac" reason:err.localizedDescription userInfo:nil]; }
    return r;
}

- (NSNumber *)inboundGroupSessionClose:(double)handle {
    return [VodozemacImpl inboundGroupSessionClose:@(handle)];
}

#pragma mark - TurboModule glue

// Returns the JSI binding for this module — RN's codegen requires this.
- (std::shared_ptr<facebook::react::TurboModule>)getTurboModule:
    (const facebook::react::ObjCTurboModule::InitParams &)params {
    return std::make_shared<facebook::react::NativeVodozemacSpecJSI>(params);
}

@end

// The dynamic bridge keeps ordinary cargo builds independent of the Sparkle SDK.
// These selectors are from the pinned Sparkle 2.10.0 public headers.
#import <AppKit/AppKit.h>
#import <Foundation/Foundation.h>
#import <dispatch/dispatch.h>

@interface NSObject (BeniSparkleAPI)
- (instancetype)initWithStartingUpdater:(BOOL)start updaterDelegate:(id)delegate userDriverDelegate:(id)driver;
@property(nonatomic, readonly) NSObject *updater;
- (BOOL)startUpdater:(NSError **)error;
- (void)checkForUpdates:(id)sender;
@property(nonatomic, readonly) BOOL canCheckForUpdates;
@property(nonatomic) BOOL automaticallyChecksForUpdates;
@property(nonatomic) BOOL sendsSystemProfile;
@end

@interface BeniSparkleDelegate : NSObject
@property(nonatomic, copy) void (^pendingInstall)(void);
@property(nonatomic) NSInteger lastErrorCode;
@end
@implementation BeniSparkleDelegate
- (NSSet<NSString *> *)allowedChannelsForUpdater:(id)updater {
    NSString *channel = [NSBundle.mainBundle objectForInfoDictionaryKey:@"BeniUpdateChannel"];
    return [channel isEqualToString:@"beta"] ? [NSSet setWithObject:@"beta"] : [NSSet set];
}
- (BOOL)updater:(id)updater shouldPostponeRelaunchForUpdate:(id)item untilInvokingBlock:(void (^)(void))handler {
    // Always hand off to Rust first: save settings, stop IPC and, if necessary,
    // let the user finish or explicitly discard session-only timers/reminders.
    self.pendingInstall = handler;
    return YES;
}
- (void)updater:(id)updater didAbortWithError:(NSError *)error {
    self.pendingInstall = nil;
    self.lastErrorCode = error.code; // Fixed numeric diagnostic, no URLs or payloads.
}
@end

@interface BeniSparkleHandle : NSObject
@property(nonatomic, strong) NSBundle *framework;
@property(nonatomic, strong) NSObject *controller;
@property(nonatomic, strong) BeniSparkleDelegate *delegate;
@property(nonatomic) BOOL feedReady;
@end
@implementation BeniSparkleHandle
@end

// Handles are retained ARC objects; all entry points require the main thread.
void *beni_updater_create(int32_t *status) {
    *status = 3; // unavailable
    if (![NSThread isMainThread]) return NULL;
    NSBundle *host = NSBundle.mainBundle;
    NSString *path = [host.bundlePath stringByAppendingPathComponent:@"Contents/Frameworks/Sparkle.framework"];
    NSBundle *framework = [NSBundle bundleWithPath:path];
    if (framework == nil) { *status = 4; return NULL; }
    if (![framework loadAndReturnError:NULL]) { *status = 5; return NULL; }
    Class controllerClass = NSClassFromString(@"SPUStandardUpdaterController");
    if (controllerClass == Nil) { *status = 6; return NULL; }
    BeniSparkleHandle *handle = [BeniSparkleHandle new];
    handle.framework = framework; // Never unload classes while instances exist.
    handle.delegate = [BeniSparkleDelegate new];
    handle.controller = [[controllerClass alloc] initWithStartingUpdater:NO updaterDelegate:handle.delegate userDriverDelegate:nil];
    if (handle.controller == nil) { *status = 7; return NULL; }
    // SUEnableSystemProfiling disables the permission UI, but a stale stored
    // SUSendProfileInfo preference could still enable data collection.
    handle.controller.updater.sendsSystemProfile = NO;
    handle.feedReady = [[host objectForInfoDictionaryKey:@"BeniUpdateFeedReady"] boolValue];
    if (handle.feedReady) {
        if (![handle.controller.updater startUpdater:NULL]) { *status = 8; return NULL; }
        *status = 1; // ready
    } else {
        *status = 2; // framework ready, public feed not yet enabled; no network
    }
    return (__bridge_retained void *)handle;
}

int32_t beni_updater_can_check(void *pointer) {
    if (![NSThread isMainThread] || pointer == NULL) return 0;
    BeniSparkleHandle *handle = (__bridge BeniSparkleHandle *)pointer;
    return handle.feedReady && handle.controller.updater.canCheckForUpdates;
}

void beni_updater_check(void *pointer) {
    if (!beni_updater_can_check(pointer)) return;
    BeniSparkleHandle *handle = (__bridge BeniSparkleHandle *)pointer;
    [handle.controller checkForUpdates:nil];
}

int32_t beni_updater_automatic(void *pointer) {
    if (![NSThread isMainThread] || pointer == NULL) return 0;
    return ((__bridge BeniSparkleHandle *)pointer).controller.updater.automaticallyChecksForUpdates;
}

void beni_updater_set_automatic(void *pointer, int32_t enabled) {
    if (![NSThread isMainThread] || pointer == NULL) return;
    BeniSparkleHandle *handle = (__bridge BeniSparkleHandle *)pointer;
    if (handle.feedReady) handle.controller.updater.automaticallyChecksForUpdates = enabled != 0;
}

int32_t beni_updater_install_pending(void *pointer) {
    if (![NSThread isMainThread] || pointer == NULL) return 0;
    return ((__bridge BeniSparkleHandle *)pointer).delegate.pendingInstall != nil;
}

void beni_updater_finish_install(void *pointer) {
    if (![NSThread isMainThread] || pointer == NULL) return;
    BeniSparkleHandle *handle = (__bridge BeniSparkleHandle *)pointer;
    void (^handler)(void) = handle.delegate.pendingInstall;
    handle.delegate.pendingInstall = nil;
    // Never terminate from a nested Rust/egui callback. Run after it returns.
    if (handler != nil) dispatch_async(dispatch_get_main_queue(), handler);
}

void beni_updater_destroy(void *pointer) {
    if (![NSThread isMainThread] || pointer == NULL) return;
    BeniSparkleHandle *handle = (__bridge_transfer BeniSparkleHandle *)pointer;
    handle.delegate.pendingInstall = nil;
}

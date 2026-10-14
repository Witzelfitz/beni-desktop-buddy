// Local probe only. Compile into a disposable bundle, never ship this driver.
#include "../../apps/desktop/src/updater/macos.m"
#include <stdio.h>

@interface NSObject (BeniProbeHost)
@property(nonatomic, readonly) NSBundle *hostBundle;
@end


#define CHECK(condition) do { if (!(condition)) { fprintf(stderr, "FAIL: native updater probe line %d\n", __LINE__); return 2; } } while (0)

int main(int argc, char **argv) {
    @autoreleasepool {
        CHECK(argc == 2);
        [NSApplication sharedApplication];
        [NSApp finishLaunching];
        NSUserDefaults *testDefaults = [[NSUserDefaults alloc] initWithSuiteName:[NSBundle.mainBundle objectForInfoDictionaryKey:@"SUDefaultsDomain"]];
        [testDefaults setBool:YES forKey:@"SUSendProfileInfo"];
        int32_t status = 0;
        void *pointer = beni_updater_create(&status);
        if (pointer == NULL) fprintf(stderr, "FAIL: native updater create status %d\n", status);
        CHECK(pointer != NULL);
        BeniSparkleHandle *handle = (__bridge BeniSparkleHandle *)pointer;
        CHECK(!handle.controller.updater.sendsSystemProfile);
        CHECK([handle.controller.updater.hostBundle.bundleIdentifier isEqualToString:NSBundle.mainBundle.bundleIdentifier]);
        CHECK([[handle.controller.updater.hostBundle objectForInfoDictionaryKey:@"SUDefaultsDomain"] isEqualToString:[NSBundle.mainBundle objectForInfoDictionaryKey:@"SUDefaultsDomain"]]);
        BOOL beta = [[NSBundle.mainBundle objectForInfoDictionaryKey:@"BeniUpdateChannel"] isEqualToString:@"beta"];
        CHECK([[handle.delegate allowedChannelsForUpdater:handle.controller.updater] containsObject:@"beta"] == beta);
        NSString *mode = [NSString stringWithUTF8String:argv[1]];
        if ([mode isEqualToString:@"pending"]) {
            CHECK(status == 2);
            CHECK(beni_updater_can_check(pointer) == 0);
            CHECK(beni_updater_automatic(pointer) == 0);
            beni_updater_set_automatic(pointer, 1);
            CHECK(beni_updater_automatic(pointer) == 0);
        } else {
            CHECK(status == 1);
            CHECK(beni_updater_can_check(pointer) != 0);
            if ([mode isEqualToString:@"enable"]) {
                CHECK(beni_updater_automatic(pointer) == 0);
                beni_updater_set_automatic(pointer, 1);
                CHECK(beni_updater_automatic(pointer) != 0);
                NSUserDefaults *suite = [[NSUserDefaults alloc] initWithSuiteName:[NSBundle.mainBundle objectForInfoDictionaryKey:@"SUDefaultsDomain"]];
                CHECK([suite boolForKey:@"SUEnableAutomaticChecks"]);
                [suite synchronize];
            } else if ([mode isEqualToString:@"persisted"]) {
                CHECK(beni_updater_automatic(pointer) != 0);
                beni_updater_set_automatic(pointer, 0);
                [[[NSUserDefaults alloc] initWithSuiteName:[NSBundle.mainBundle objectForInfoDictionaryKey:@"SUDefaultsDomain"]] synchronize];
            } else if ([mode isEqualToString:@"gate"]) {
                CHECK(beni_updater_install_pending(pointer) == 0);
                __block int calls = 0;
                CHECK([handle.delegate updater:handle.controller.updater shouldPostponeRelaunchForUpdate:nil untilInvokingBlock:^{ calls++; }] == YES);
                CHECK(beni_updater_install_pending(pointer) != 0);
                CHECK(calls == 0);
                beni_updater_finish_install(pointer);
                CHECK(beni_updater_install_pending(pointer) == 0);
                CHECK(calls == 0); // No reentrant termination from Rust callbacks.
                [NSRunLoop.currentRunLoop runUntilDate:[NSDate dateWithTimeIntervalSinceNow:0.1]];
                CHECK(calls == 1);
                beni_updater_finish_install(pointer);
                [NSRunLoop.currentRunLoop runUntilDate:[NSDate dateWithTimeIntervalSinceNow:0.1]];
                CHECK(calls == 1);
            } else if ([mode isEqualToString:@"bad-feed"]) {
                CHECK(beni_updater_automatic(pointer) == 0);
                // Sparkle completes an error cycle after the user dismisses its
                // own alert. Only dismiss this process's modal; no global AX.
                __block BOOL sawErrorDialog = NO;
                NSTimer *dismiss = [NSTimer timerWithTimeInterval:0.05 repeats:YES block:^(NSTimer *timer) {
                    if (NSApp.modalWindow != nil) {
                        sawErrorDialog = YES;
                        [NSApp stopModalWithCode:NSAlertFirstButtonReturn];
                    }
                }];
                [NSRunLoop.currentRunLoop addTimer:dismiss forMode:NSDefaultRunLoopMode];
                [NSRunLoop.currentRunLoop addTimer:dismiss forMode:NSRunLoopCommonModes];
                [NSRunLoop.currentRunLoop addTimer:dismiss forMode:NSModalPanelRunLoopMode];
                NSDate *deadline = [NSDate dateWithTimeIntervalSinceNow:10];
                NSTimer *watch = [NSTimer scheduledTimerWithTimeInterval:0.05 repeats:YES block:^(NSTimer *timer) {
                    if (handle.delegate.lastErrorCode != 0 || deadline.timeIntervalSinceNow <= 0) {
                        [NSApp stop:nil];
                        [NSApp postEvent:[NSEvent otherEventWithType:NSEventTypeApplicationDefined location:NSZeroPoint modifierFlags:0 timestamp:0 windowNumber:0 context:nil subtype:0 data1:0 data2:0] atStart:NO];
                    }
                }];
                beni_updater_check(pointer);
                [NSApp run];
                [watch invalidate];
                CHECK(handle.delegate.lastErrorCode != 0);
                CHECK(sawErrorDialog);
                CHECK(beni_updater_install_pending(pointer) == 0);
                [dismiss invalidate];
            } else {
                CHECK(NO);
            }
        }
        __block int32_t backgroundStatus = 0;
        __block void *background = NULL;
        dispatch_semaphore_t done = dispatch_semaphore_create(0);
        // dispatch_sync may execute inline on the main thread; async plus a
        // semaphore is needed to exercise the actual background-thread guard.
        dispatch_async(dispatch_get_global_queue(QOS_CLASS_DEFAULT, 0), ^{
            background = beni_updater_create(&backgroundStatus);
            dispatch_semaphore_signal(done);
        });
        CHECK(dispatch_semaphore_wait(done, dispatch_time(DISPATCH_TIME_NOW, NSEC_PER_SEC)) == 0);
        CHECK(background == NULL && backgroundStatus == 3);
        beni_updater_destroy(pointer);
        printf("PASS: native Sparkle bridge (%s)\n", argv[1]);
    }
    return 0;
}

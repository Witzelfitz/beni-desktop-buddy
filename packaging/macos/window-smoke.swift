import CoreGraphics
import Foundation

guard CommandLine.arguments.count == 2,
      let pid = Int32(CommandLine.arguments[1]),
      let windows = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] else {
    exit(2)
}
// Query metadata only for the test process; no screenshots or accessibility permissions.
let visible = windows.contains { window in
    let layer = (window[kCGWindowLayer as String] as? NSNumber)?.intValue
    let bounds = window[kCGWindowBounds as String] as? [String: Any]
    let width = (bounds?["Width"] as? NSNumber)?.doubleValue ?? 0
    let height = (bounds?["Height"] as? NSNumber)?.doubleValue ?? 0
    // Beni normally floats at layer 3; exclude its tiny menu-bar window.
    return (window[kCGWindowOwnerPID as String] as? NSNumber)?.int32Value == pid
        && (layer == 0 || layer == 3) && width >= 100 && height >= 100
}
exit(visible ? 0 : 1)

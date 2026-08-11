import Darwin
import Foundation
import ImageIO
import ScreenCaptureKit

@main
struct WindowCapture {
  static func fail(_ message: String, _ code: Int32) -> Never {
    FileHandle.standardError.write(Data("\(message)\n".utf8))
    exit(code)
  }

  static func main() async {
    do {
      try await capture()
    } catch {
      fail(error.localizedDescription, 1)
    }
  }

  static func capture() async throws {
    guard CommandLine.arguments.count == 5,
          let windowId = UInt32(CommandLine.arguments[1]),
          let width = Int(CommandLine.arguments[2]),
          let height = Int(CommandLine.arguments[3]) else {
      fail("usage: capture WINDOW_ID WIDTH HEIGHT OUTPUT", 2)
    }
    let output = CommandLine.arguments[4]
    let content = try await SCShareableContent.excludingDesktopWindows(false, onScreenWindowsOnly: true)
    guard let window = content.windows.first(where: { $0.windowID == windowId }) else {
      fail("ScreenCaptureKit could not find window \(windowId)", 3)
    }
    let filter = SCContentFilter(desktopIndependentWindow: window)
    let configuration = SCStreamConfiguration()
    configuration.width = width
    configuration.height = height
    configuration.showsCursor = false
    let image = try await SCScreenshotManager.captureImage(contentFilter: filter, configuration: configuration)
    guard let destination = CGImageDestinationCreateWithURL(URL(fileURLWithPath: output) as CFURL, "public.png" as CFString, 1, nil) else {
      fail("could not create PNG destination", 4)
    }
    CGImageDestinationAddImage(destination, image, nil)
    guard CGImageDestinationFinalize(destination) else {
      fail("could not finalize PNG destination", 5)
    }
  }
}

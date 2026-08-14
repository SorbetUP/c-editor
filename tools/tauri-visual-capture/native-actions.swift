import CoreGraphics
import Foundation
import AppKit

struct ActionRequest: Codable {
  let operation: String
  let processId: Int32?
  let points: [[Double]]?
  let key: String?
  let repeatCount: Int?
  let text: String?
  let deltaY: Int32?
  let control: Bool?
  let command: Bool?
}
struct ActionResult: Codable {
  let ok: Bool
  let operation: String
  let eventCount: Int
  let accessibilityTrusted: Bool
  let dispatchedAt: [String]
}

func point(_ values: [Double]) -> CGPoint {
  CGPoint(x: values.first ?? 0, y: values.dropFirst().first ?? 0)
}

let eventSource = CGEventSource(stateID: .hidSystemState)

func postMouse(_ type: CGEventType, at position: CGPoint, button: CGMouseButton = .left) {
  guard let event = CGEvent(mouseEventSource: eventSource, mouseType: type, mouseCursorPosition: position, mouseButton: button) else {
    fatalError("CGEvent could not create a mouse event")
  }
  if type == .leftMouseDown || type == .leftMouseUp {
    event.setIntegerValueField(.mouseEventClickState, value: 1)
  }
  event.post(tap: .cghidEventTap)
}

func keyCode(_ key: String) -> CGKeyCode {
  switch key {
  case "Escape": return 53
  case "Enter", "Return": return 36
  case "Tab": return 48
  case "Space": return 49
  case "Backspace": return 51
  case "Control+End": return 119
  case "End": return 119
  default: return 0
  }
}

func postKey(_ name: String, control: Bool, command: Bool) {
  let code = keyCode(name)
  let flags: CGEventFlags = [control ? .maskControl : [], command ? .maskCommand : []]
  guard let down = CGEvent(keyboardEventSource: eventSource, virtualKey: code, keyDown: true),
        let up = CGEvent(keyboardEventSource: eventSource, virtualKey: code, keyDown: false) else {
    fatalError("CGEvent could not create a keyboard event")
  }
  down.flags = flags
  up.flags = flags
  down.post(tap: .cghidEventTap)
  usleep(12_000)
  up.post(tap: .cghidEventTap)
}

func postText(_ value: String) {
  guard let event = CGEvent(keyboardEventSource: eventSource, virtualKey: 0, keyDown: true) else {
    fatalError("CGEvent could not create a text event")
  }
  var utf16 = Array(value.utf16)
  event.keyboardSetUnicodeString(stringLength: utf16.count, unicodeString: &utf16)
  event.post(tap: .cghidEventTap)
  usleep(12_000)
  if let up = CGEvent(keyboardEventSource: eventSource, virtualKey: 0, keyDown: false) {
    up.post(tap: .cghidEventTap)
  }
}

func activateProcess(_ processId: Int32?) {
  guard let processId,
        let application = NSRunningApplication(processIdentifier: pid_t(processId)) else { return }
  application.activate(options: [.activateIgnoringOtherApps])
  usleep(80_000)
}

func dispatch(_ request: ActionRequest) -> Int {
  activateProcess(request.processId)
  let points = (request.points ?? []).map(point)
  switch request.operation {
  case "move-pointer":
    for position in points {
      postMouse(.mouseMoved, at: position)
      usleep(50_000)
    }
  case "click":
    guard let position = points.first else { fatalError("click requires a point") }
    postMouse(.mouseMoved, at: position)
    postMouse(.leftMouseDown, at: position)
    usleep(20_000)
    postMouse(.leftMouseUp, at: position)
  case "drag":
    guard let first = points.first else { fatalError("drag requires points") }
    postMouse(.mouseMoved, at: first)
    postMouse(.leftMouseDown, at: first)
    for position in points.dropFirst() {
      usleep(50_000)
      postMouse(.leftMouseDragged, at: position)
    }
    if let last = points.last { postMouse(.leftMouseUp, at: last) }
  case "press-key":
    let count = max(1, request.repeatCount ?? 1)
    for _ in 0..<count { postKey(request.key ?? "", control: request.control ?? false, command: request.command ?? false) }
  case "write-text":
    postText(request.text ?? "")
  case "scroll":
    guard let position = points.first else { fatalError("scroll requires a point") }
    postMouse(.mouseMoved, at: position)
    guard let event = CGEvent(scrollWheelEvent2Source: eventSource, units: .pixel, wheelCount: 1, wheel1: request.deltaY ?? 0, wheel2: 0, wheel3: 0) else {
      fatalError("CGEvent could not create a scroll event")
    }
    event.post(tap: .cghidEventTap)
  default:
    fatalError("unsupported physical operation: \(request.operation)")
  }
  return points.count
}

let requestPath = CommandLine.arguments.dropFirst().first
guard let requestPath, let data = FileManager.default.contents(atPath: requestPath) else {
  fputs("native-actions requires a JSON request file\n", stderr)
  exit(2)
}
do {
  let request = try JSONDecoder().decode(ActionRequest.self, from: data)
  if #available(macOS 10.15, *), !CGPreflightPostEventAccess() {
    fputs("macOS Accessibility event posting is not trusted for this process\n", stderr)
    exit(3)
  }
  let count = dispatch(request)
  let result = ActionResult(
    ok: true,
    operation: request.operation,
    eventCount: count,
    accessibilityTrusted: CGPreflightPostEventAccess(),
    dispatchedAt: [ISO8601DateFormatter().string(from: Date())]
  )
  print(String(data: try JSONEncoder().encode(result), encoding: .utf8)!)
} catch {
  fputs("native-actions: \(error)\n", stderr)
  exit(1)
}

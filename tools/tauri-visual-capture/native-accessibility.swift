import ApplicationServices
import CoreGraphics
import Foundation

struct Rect: Codable {
  let x: Double
  let y: Double
  let width: Double
  let height: Double
}

struct Element: Codable {
  let role: String
  let subrole: String
  let title: String
  let description: String
  let value: String
  let rect: Rect?
  let depth: Int
}

struct Report: Codable {
  let accessibilityTrusted: Bool
  let elements: [Element]
}

func attribute(_ element: AXUIElement, _ key: CFString) -> CFTypeRef? {
  var value: CFTypeRef?
  guard AXUIElementCopyAttributeValue(element, key, &value) == .success else { return nil }
  return value
}

func text(_ value: CFTypeRef?) -> String {
  if let value = value as? String { return value }
  if let value = value as? NSNumber { return value.stringValue }
  return ""
}

func rect(_ value: CFTypeRef?) -> Rect? {
  guard let value else { return nil }
  let axValue = value as! AXValue
  switch AXValueGetType(axValue) {
  case .cgPoint:
    var point = CGPoint.zero
    guard AXValueGetValue(axValue, .cgPoint, &point) else { return nil }
    return Rect(x: point.x, y: point.y, width: 0, height: 0)
  case .cgSize:
    var size = CGSize.zero
    guard AXValueGetValue(axValue, .cgSize, &size) else { return nil }
    return Rect(x: 0, y: 0, width: size.width, height: size.height)
  default:
    return nil
  }
}

func frame(_ element: AXUIElement) -> Rect? {
  var position = CGPoint.zero
  var size = CGSize.zero
  guard let positionValue = attribute(element, kAXPositionAttribute as CFString),
        let sizeValue = attribute(element, kAXSizeAttribute as CFString),
        AXValueGetValue(positionValue as! AXValue, .cgPoint, &position),
        AXValueGetValue(sizeValue as! AXValue, .cgSize, &size) else { return nil }
  return Rect(x: position.x, y: position.y, width: size.width, height: size.height)
}

func walk(_ element: AXUIElement, depth: Int, output: inout [Element]) {
  if depth > 24 { return }
  let item = Element(
    role: text(attribute(element, kAXRoleAttribute as CFString)),
    subrole: text(attribute(element, kAXSubroleAttribute as CFString)),
    title: text(attribute(element, kAXTitleAttribute as CFString)),
    description: text(attribute(element, kAXDescriptionAttribute as CFString)),
    value: text(attribute(element, kAXValueAttribute as CFString)),
    rect: frame(element),
    depth: depth
  )
  output.append(item)
  guard let children = attribute(element, kAXChildrenAttribute as CFString) as? [AXUIElement] else { return }
  for child in children { walk(child, depth: depth + 1, output: &output) }
}

guard let pid = Int32(CommandLine.arguments.dropFirst().first ?? "") else {
  fputs("native-accessibility requires a process id\n", stderr)
  exit(2)
}

let application = AXUIElementCreateApplication(pid)
guard let windows = attribute(application, kAXWindowsAttribute as CFString) as? [AXUIElement] else {
  let report = Report(accessibilityTrusted: AXIsProcessTrusted(), elements: [])
  print(String(data: try! JSONEncoder().encode(report), encoding: .utf8)!)
  exit(0)
}

var elements: [Element] = []
for window in windows { walk(window, depth: 0, output: &elements) }
let report = Report(accessibilityTrusted: AXIsProcessTrusted(), elements: elements)
print(String(data: try! JSONEncoder().encode(report), encoding: .utf8)!)

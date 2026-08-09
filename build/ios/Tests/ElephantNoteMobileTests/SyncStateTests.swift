import XCTest
@testable import ElephantNoteMobile

final class SyncStateTests: XCTestCase {
    func testCreatesStableDeviceIdentity() {
        let defaults = UserDefaults(suiteName: "ElephantNoteMobileTests-\(UUID().uuidString)")!

        let first = SyncState.load(defaults: defaults)
        let second = SyncState.load(defaults: defaults)

        XCTAssertTrue(first.deviceId.hasPrefix("en-"))
        XCTAssertEqual(first, second)
    }
}

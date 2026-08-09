import Foundation

public struct SyncState: Codable, Equatable {
    public var deviceId: String
    public var folderId: String
    public var remote: String

    public init(deviceId: String, folderId: String = "vault-mobile", remote: String = "") {
        self.deviceId = deviceId
        self.folderId = folderId
        self.remote = remote
    }

    public static func load(defaults: UserDefaults = .standard) -> SyncState {
        if let data = defaults.data(forKey: "elephantnote.sync"),
           let state = try? JSONDecoder().decode(SyncState.self, from: data) {
            return state
        }
        let state = SyncState(deviceId: "en-\(UUID().uuidString.replacingOccurrences(of: "-", with: "").prefix(12))")
        state.save(defaults: defaults)
        return state
    }

    public func save(defaults: UserDefaults = .standard) {
        if let data = try? JSONEncoder().encode(self) {
            defaults.set(data, forKey: "elephantnote.sync")
        }
    }
}

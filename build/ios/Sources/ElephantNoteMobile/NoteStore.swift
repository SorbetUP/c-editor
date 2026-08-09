import Combine
import Foundation

public struct OfflineNote: Codable, Identifiable, Equatable {
    public var id: String
    public var title: String
    public var body: String
    public var updatedAt: Date
}

public final class NoteStore: ObservableObject {
    @Published public private(set) var notes: [OfflineNote] = []
    private let defaults: UserDefaults

    public init(defaults: UserDefaults = .standard) {
        self.defaults = defaults
        self.notes = Self.load(defaults: defaults)
    }

    public func create(title: String, body: String) {
        let cleanTitle = title.trimmingCharacters(in: .whitespacesAndNewlines)
        notes.insert(OfflineNote(
            id: "note-\(Int(Date().timeIntervalSince1970 * 1000))",
            title: cleanTitle.isEmpty ? "Untitled" : cleanTitle,
            body: body,
            updatedAt: Date()
        ), at: 0)
        save()
    }

    private func save() {
        if let data = try? JSONEncoder().encode(notes) {
            defaults.set(data, forKey: "elephantnote.notes")
        }
    }

    private static func load(defaults: UserDefaults) -> [OfflineNote] {
        guard let data = defaults.data(forKey: "elephantnote.notes"),
              let notes = try? JSONDecoder().decode([OfflineNote].self, from: data) else {
            return []
        }
        return notes
    }
}

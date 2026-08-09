import SwiftUI

public struct ElephantNoteMobileApp: App {
    public init() {}

    public var body: some Scene {
        WindowGroup {
            NotesView()
        }
    }
}

public struct NotesView: View {
    @StateObject private var store = NoteStore()
    @State private var syncState = SyncState.load()
    @State private var draft = ""

    public init() {}

    public var body: some View {
        NavigationStack {
            List {
                Section {
                    Text(syncState.deviceId + " / " + syncState.folderId)
                        .font(.footnote)
                        .foregroundStyle(.secondary)
                    TextEditor(text: $draft)
                        .frame(minHeight: 120)
                    Button("Save offline") {
                        let firstLine = draft.components(separatedBy: .newlines).first ?? "Untitled"
                        store.create(title: firstLine, body: draft)
                        draft = ""
                    }
                    .disabled(draft.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                }
                Section {
                    ForEach(store.notes) { note in
                        VStack(alignment: .leading, spacing: 4) {
                            Text(note.title).font(.headline)
                            Text(note.body).font(.body)
                            Text(note.updatedAt.formatted()).font(.caption).foregroundStyle(.secondary)
                        }
                    }
                }
            }
            .navigationTitle("ElephantNote")
        }
    }
}

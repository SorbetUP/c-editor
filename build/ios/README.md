# ElephantNote iOS

Minimal SwiftUI scaffold for offline capture and future Git-backed sync.

Implemented:

- local notes through `UserDefaults`
- stable sync identity (`deviceId`, `folderId`, optional remote)
- a compact Swift Package that can be opened in Xcode and embedded in an iOS app target

Smoke check:

```bash
swift test --package-path ios
```

To ship an `.ipa`, create an Xcode iOS App target named `ElephantNoteMobile`,
add the `ElephantNoteMobile` package product, and use the SwiftUI
`ElephantNoteMobileApp` entry point from `Sources/ElephantNoteMobile`.
iOS signing and provisioning are intentionally outside this repository.

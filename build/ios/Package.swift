// swift-tools-version: 5.10
import PackageDescription

let package = Package(
    name: "ElephantNoteMobile",
    platforms: [.iOS(.v16), .macOS(.v13)],
    products: [
        .library(name: "ElephantNoteMobile", targets: ["ElephantNoteMobile"])
    ],
    targets: [
        .target(name: "ElephantNoteMobile"),
        .testTarget(name: "ElephantNoteMobileTests", dependencies: ["ElephantNoteMobile"])
    ]
)

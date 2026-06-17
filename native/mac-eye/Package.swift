// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "mac-eye",
    platforms: [
        .macOS(.v14)
    ],
    targets: [
        .executableTarget(
            name: "mac-eye",
            path: "Sources/mac-eye"
        )
    ]
)

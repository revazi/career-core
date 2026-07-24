// swift-tools-version: 5.9

import PackageDescription

let package = Package(
    name: "CareerCoreSwift",
    platforms: [
        .macOS(.v13),
        .iOS(.v16),
    ],
    products: [
        .library(name: "CareerCore", targets: ["CareerCore"]),
        .executable(name: "career-core-smoke", targets: ["CareerCoreSmoke"]),
    ],
    targets: [
        .binaryTarget(
            name: "CareerCoreFFI",
            path: "Artifacts/CareerCoreFFI.xcframework"
        ),
        .target(
            name: "CareerCore",
            dependencies: ["CareerCoreFFI"]
        ),
        .executableTarget(
            name: "CareerCoreSmoke",
            dependencies: ["CareerCore"]
        ),
        .testTarget(
            name: "CareerCoreTests",
            dependencies: ["CareerCore"]
        ),
    ]
)

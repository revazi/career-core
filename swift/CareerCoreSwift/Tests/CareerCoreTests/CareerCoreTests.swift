import CareerCore
import Foundation
import XCTest

final class CareerCoreTests: XCTestCase {
    private typealias Operation = (String) throws -> String

    private static let repositoryRoot: URL = {
        var url = URL(fileURLWithPath: #filePath)
        for _ in 0..<5 {
            url.deleteLastPathComponent()
        }
        return url
    }()

    func testCapabilitiesAreVersionedJSON() throws {
        let output = try capabilitiesJson()
        let document = try XCTUnwrap(
            JSONSerialization.jsonObject(with: Data(output.utf8)) as? [String: Any]
        )

        XCTAssertEqual(document["schema_version"] as? String, "career.capabilities.v1")
        XCTAssertTrue(output.hasSuffix("\n"))
    }

    func testEveryOperationMatchesCanonicalRustGolden() throws {
        let fixtures: [(String, String, Operation)] = [
            (
                "fixtures/resume/phase1/complete-sections.input.json",
                "fixtures/resume/phase1/complete-sections.expected.json",
                resumeEvaluateJson
            ),
            (
                "fixtures/resume/phase3/complete-analysis.input.json",
                "fixtures/resume/phase3/complete-analysis.expected.json",
                resumeAnalyzeJson
            ),
            (
                "fixtures/resume/phase7/complete-analysis-suggestion-review.input.json",
                "fixtures/resume/phase7/complete-analysis-suggestion-review.expected.json",
                resumeAnalysisSuggestionsReviewJson
            ),
            (
                "fixtures/resume/phase2/complete-normalization.input.json",
                "fixtures/resume/phase2/complete-normalization.expected.json",
                resumeNormalizeJson
            ),
            (
                "fixtures/resume/phase2/messy-unlabeled.enrichment-input.json",
                "fixtures/resume/phase2/messy-unlabeled.enrichment-expected.json",
                resumeEnrichJson
            ),
            (
                "fixtures/resume/phase7/complete-variant-review.input.json",
                "fixtures/resume/phase7/complete-variant-review.expected.json",
                resumeVariantReviewJson
            ),
            (
                "fixtures/resume/phase7/selected-variant-materialization.input.json",
                "fixtures/resume/phase7/selected-variant-materialization.expected.json",
                resumeVariantMaterializeJson
            ),
            (
                "fixtures/job/phase4a/complete-normalization.input.json",
                "fixtures/job/phase4a/complete-normalization.expected.json",
                jobNormalizeJson
            ),
            (
                "fixtures/job/phase4b/complete-match.input.json",
                "fixtures/job/phase4b/complete-match.expected.json",
                jobMatchJson
            ),
        ]

        for (inputPath, expectedPath, operation) in fixtures {
            let input = try fixture(inputPath)
            let expected = try fixture(expectedPath)
            XCTAssertEqual(try operation(input), expected, "fixture changed: \(inputPath)")
        }
    }

    func testMalformedJSONMapsToTypedSwiftError() {
        XCTAssertThrowsError(
            try resumeEvaluateJson(
                inputJson: #"{"schema_version":"career.resume_input.v1","text":}"#
            )
        ) { error in
            guard case let CareerSwiftError.InvalidJson(message) = error else {
                return XCTFail("unexpected error: \(error)")
            }
            XCTAssertTrue(message.contains("career.resume_input.v1"))
            XCTAssertFalse(message.contains("private resume payload"))
        }
    }

    func testCoreValidationMapsCodeMessageAndFieldPath() {
        XCTAssertThrowsError(
            try resumeEvaluateJson(
                inputJson: #"{"schema_version":"career.resume_input.v1","text":" "}"#
            )
        ) { error in
            guard case let CareerSwiftError.InvalidInput(code, message, fieldPath) = error else {
                return XCTFail("unexpected error: \(error)")
            }
            XCTAssertEqual(code, "source_text_empty")
            XCTAssertEqual(fieldPath, "text")
            XCTAssertFalse(message.contains("private resume payload"))
        }
    }

    func testAdapterRejectsOversizedJSONBeforeParsing() {
        let oversized = String(repeating: "x", count: 262_145)
        XCTAssertThrowsError(try resumeEvaluateJson(inputJson: oversized)) { error in
            guard case let CareerSwiftError.InputTooLarge(actualBytes, maximumBytes) = error else {
                return XCTFail("unexpected error: \(error)")
            }
            XCTAssertEqual(actualBytes, 262_145)
            XCTAssertEqual(maximumBytes, 262_144)
        }
    }

    private func fixture(_ relativePath: String) throws -> String {
        try String(
            contentsOf: Self.repositoryRoot.appendingPathComponent(relativePath),
            encoding: .utf8
        )
    }
}

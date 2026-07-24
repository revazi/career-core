import CareerCore
import Foundation

let output = try capabilitiesJson()
FileHandle.standardOutput.write(Data(output.utf8))

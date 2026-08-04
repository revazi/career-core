#!/usr/bin/env node

import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { access, mkdtemp, readFile, rm } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { pathToFileURL, fileURLToPath } from "node:url";

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const packageManifest = JSON.parse(await readFile(path.join(repositoryRoot, "package.json"), "utf8"));

function piPackageRoot() {
  const explicit = process.env.PI_CODING_AGENT_PACKAGE_PATH;
  if (explicit) {
    if (!path.isAbsolute(explicit)) throw new Error("PI_CODING_AGENT_PACKAGE_PATH must be absolute");
    return explicit;
  }
  const globalRoot = execFileSync("npm", ["root", "--global"], {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "ignore"],
  }).trim();
  return path.join(globalRoot, "@earendil-works", "pi-coding-agent");
}

const piRoot = piPackageRoot();
const piManifest = JSON.parse(await readFile(path.join(piRoot, "package.json"), "utf8"));
assert.equal(piManifest.name, "@earendil-works/pi-coding-agent");
const pi = await import(pathToFileURL(path.join(piRoot, "dist", "index.js")).href);
const extensionPaths = packageManifest.pi.extensions.map((entry) => path.resolve(repositoryRoot, entry));
const skillPaths = packageManifest.pi.skills.map((entry) => path.resolve(repositoryRoot, entry));

assert.deepEqual(packageManifest.pi.skills, ["./.agents/skills/career-core"]);
assert.equal(packageManifest.dependencies, undefined);
assert.deepEqual(Object.keys(packageManifest.peerDependencies).sort(), [
  "@earendil-works/pi-ai",
  "@earendil-works/pi-coding-agent",
  "typebox",
]);

const temporaryRoot = await mkdtemp(path.join(os.tmpdir(), "career-core-pi-load-"));
const cwd = path.join(temporaryRoot, "cwd");
const agentDir = path.join(temporaryRoot, "agent");
let networkAttempted = false;
const originalFetch = globalThis.fetch;
globalThis.fetch = async () => {
  networkAttempted = true;
  throw new Error("network access is forbidden in the no-model load smoke");
};
process.env.PI_OFFLINE = "1";

try {
  const settingsManager = pi.SettingsManager.inMemory({ packages: [] });
  const loader = new pi.DefaultResourceLoader({
    cwd,
    agentDir,
    settingsManager,
    additionalExtensionPaths: extensionPaths,
    additionalSkillPaths: skillPaths,
    noPromptTemplates: true,
    noThemes: true,
    noContextFiles: true,
  });
  await loader.reload();

  const extensionResult = loader.getExtensions();
  assert.deepEqual(extensionResult.errors, []);
  const packageExtensions = extensionResult.extensions.filter((extension) =>
    extensionPaths.includes(path.resolve(extension.path)),
  );
  assert.equal(packageExtensions.length, 1);

  const registeredTools = [...packageExtensions[0].tools.values()].map(({ definition }) => definition);
  const names = registeredTools.map(({ name }) => name).sort();
  assert.deepEqual(names, ["career_core_discover", "career_core_job", "career_core_resume"]);

  for (const tool of registeredTools) {
    assert.match(tool.name, /^[a-z][a-z0-9_]{0,63}$/);
    assert.equal(tool.parameters.type, "object");
    assert.equal(tool.parameters.additionalProperties, false);
    assert.ok(Array.isArray(tool.parameters.required));
    assert.ok(tool.parameters.properties.operation);
  }

  const schemas = new Map(registeredTools.map((tool) => [tool.name, tool.parameters]));
  assert.deepEqual(schemas.get("career_core_discover").required, ["operation"]);
  assert.deepEqual(schemas.get("career_core_resume").required.sort(), ["input_json", "operation"]);
  assert.deepEqual(schemas.get("career_core_job").required.sort(), ["input_json", "operation"]);

  const serializedSchemas = JSON.stringify(Object.fromEntries(schemas));
  for (const operation of [
    "capabilities",
    "schema-list",
    "schema-export",
    "evaluate",
    "analyze",
    "analysis-suggestions-review",
    "analysis-replacements-review",
    "normalize",
    "enrich",
    "variant-review",
    "variant-materialize",
    "match",
  ]) {
    assert.ok(serializedSchemas.includes(`"${operation}"`));
  }

  const { skills, diagnostics } = loader.getSkills();
  assert.deepEqual(diagnostics, []);
  const careerSkills = skills.filter((skill) => skill.name === "career-core");
  assert.equal(careerSkills.length, 1);
  assert.equal(path.resolve(careerSkills[0].filePath), path.join(skillPaths[0], "SKILL.md"));

  assert.equal(networkAttempted, false);
  await assert.rejects(access(path.join(agentDir, "auth.json")));
  process.stdout.write(`Loaded tools: ${names.join(", ")}\nLoaded skill: career-core\n`);
} finally {
  globalThis.fetch = originalFetch;
  await rm(temporaryRoot, { recursive: true, force: true });
}

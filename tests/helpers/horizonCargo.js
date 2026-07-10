const { spawnSync } = require("node:child_process");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..", "..");

function runCargo(args) {
    const result = spawnSync("cargo", args, {
        cwd: repoRoot,
        encoding: "utf8",
    });

    if (result.status !== 0) {
        throw new Error(
            [
                `cargo ${args.join(" ")} failed with status ${result.status}`,
                `stdout:\n${result.stdout}`,
                `stderr:\n${result.stderr}`,
            ].join("\n"),
        );
    }

    return result;
}

function runRustTest(testName) {
    return runCargo(["test", "--test", "horizon_flow", testName, "--", "--exact"]);
}

function cargoMetadata() {
    const result = runCargo(["metadata", "--format-version=1", "--no-deps"]);
    return JSON.parse(result.stdout);
}

function readProjectFile(...segments) {
    return fs.readFileSync(path.join(repoRoot, ...segments), "utf8");
}

function projectPath(...segments) {
    return path.join(repoRoot, ...segments);
}

module.exports = {
    cargoMetadata,
    projectPath,
    readProjectFile,
    repoRoot,
    runCargo,
    runRustTest,
};

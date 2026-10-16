const test = require("node:test");
const assert = require("node:assert/strict");

const { runCargo } = require("../helpers/horizonCargo");

test("cargo check valida la libreria", () => {
    const result = runCargo(["check"]);

    assert.match(result.stderr, /Finished/);
});

test("cargo test ejecuta la suite Rust completa", () => {
    const result = runCargo(["test"]);

    assert.match(result.stdout, /6 passed/);
    assert.match(result.stdout, /8 passed/);
    assert.match(result.stdout, /test result: ok/);
});

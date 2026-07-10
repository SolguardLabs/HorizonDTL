const test = require("node:test");
const assert = require("node:assert/strict");

const { runRustTest } = require("../helpers/horizonCargo");

test("rechaza ticket repetido", () => {
    const result = runRustTest("ticket_repetido_es_rechazado");

    assert.match(result.stdout, /test ticket_repetido_es_rechazado \.\.\. ok/);
});

test("rechaza ticket con epoca antigua", () => {
    const result = runRustTest("ticket_antiguo_es_rechazado");

    assert.match(result.stdout, /test ticket_antiguo_es_rechazado \.\.\. ok/);
});

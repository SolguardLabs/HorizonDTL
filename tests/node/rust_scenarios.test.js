const test = require("node:test");
const assert = require("node:assert/strict");

const { runRustTest } = require("../helpers/horizonCargo");

const scenarios = [
    "fixture_inicializa_superficie_operativa",
    "issue_note_bloquea_notional_en_vault_origen",
    "settle_local_distribuye_valor_y_cierra_notional",
    "settle_enrutado_actualiza_vault_de_reserva",
    "settle_enrutado_usa_indice_actual_del_vault_pagador",
];

for (const scenario of scenarios) {
    test(`escenario Rust: ${scenario}`, () => {
        const result = runRustTest(scenario);

        assert.match(result.stdout, /test result: ok/);
        assert.match(result.stdout, new RegExp(`test ${scenario} \\.\\.\\. ok`));
    });
}

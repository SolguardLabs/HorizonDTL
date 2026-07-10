const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");

const { projectPath, readProjectFile } = require("../helpers/horizonCargo");

test("usa lib.rs como punto de entrada y no define binario main.rs", () => {
    assert.equal(fs.existsSync(projectPath("src", "lib.rs")), true);
    assert.equal(fs.existsSync(projectPath("src", "main.rs")), false);
});

test("lib.rs exporta los dominios publicos principales", () => {
    const lib = readProjectFile("src", "lib.rs");

    for (const exportName of [
        "HorizonLedger",
        "DeliveryNote",
        "SignedIngressOrder",
        "RedemptionTicket",
        "SignedRedemptionTicket",
        "SettlementRoute",
        "VaultState",
        "RiskEngine",
    ]) {
        assert.match(lib, new RegExp(`pub use .*${exportName}`));
    }
});

test("la estructura de modulos separa ledger, vault, routes y tickets", () => {
    for (const directory of ["ledger", "vault", "routes", "tickets", "notes", "oracle"]) {
        assert.equal(fs.existsSync(projectPath("src", directory, "mod.rs")), true);
    }
});

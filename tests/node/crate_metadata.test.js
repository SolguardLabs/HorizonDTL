const test = require("node:test");
const assert = require("node:assert/strict");

const { cargoMetadata } = require("../helpers/horizonCargo");

test("el crate se publica como libreria Rust", () => {
    const metadata = cargoMetadata();
    const rootPackage = metadata.packages.find((pkg) => pkg.name === "horizon_dtl");

    assert.ok(rootPackage);
    assert.equal(rootPackage.version, "0.1.0");
    assert.equal(rootPackage.edition, "2024");

    const targetKinds = rootPackage.targets.map((target) => target.kind.join(","));
    assert.ok(targetKinds.includes("lib"));
    assert.equal(
        targetKinds.some((kind) => kind.includes("bin")),
        false,
    );
});

test("las dependencias principales estan declaradas", () => {
    const metadata = cargoMetadata();
    const rootPackage = metadata.packages.find((pkg) => pkg.name === "horizon_dtl");
    const dependencyNames = rootPackage.dependencies.map((dependency) => dependency.name);

    for (const name of ["blake3", "ed25519-dalek", "serde", "serde_json", "thiserror"]) {
        assert.ok(dependencyNames.includes(name), `${name} debe estar declarado`);
    }
});

const test = require("node:test");
const assert = require("node:assert/strict");

const { projectSolvency } = require("../../client/horizon");

const policy = Object.freeze({
    reserveShockBps: 1_000,
    sourceRecoveryBps: 8_000,
    backstopRecoveryBps: 5_000,
    queuedCoverageBps: 8_000,
    targetBufferBps: 1_000,
    maxRouteConcentrationBps: 5_000,
    reserveFloor: "100",
});

function input(overrides = {}) {
    return {
        payoutReserve: "1000",
        sourceReserve: "500",
        liveRoutedLiability: "800",
        queuedLocalLiability: "200",
        queuedRoutedLiability: "100",
        backstopCapacity: "100",
        largestRouteLiability: "300",
        ...overrides,
    };
}

test("proyecta el mismo waterfall entero que el nucleo", () => {
    const result = projectSolvency(policy, input());

    assert.deepEqual(result, {
        band: "normal",
        stressedPayoutReserve: "900",
        eligibleSourceReserve: "400",
        eligibleBackstop: "50",
        eligibleLiquidity: "1350",
        totalObligations: "1100",
        requiredLiquidity: "1140",
        targetLiquidity: "1254",
        capitalBuffer: "210",
        capitalShortfall: "0",
        coverageBps: "11842",
        routeConcentrationBps: "2727",
    });
    assert.equal(Object.isFrozen(result), true);
});

test("clasifica un deficit como critical", () => {
    const result = projectSolvency(policy, input({ payoutReserve: "300" }));

    assert.equal(result.band, "critical");
    assert.equal(result.capitalShortfall, "420");
});

test("clasifica una ruta concentrada como restricted", () => {
    const result = projectSolvency(policy, input({ largestRouteLiability: "700" }));

    assert.equal(result.band, "restricted");
    assert.equal(result.routeConcentrationBps, "6363");
});

test("rechaza valores negativos y basis points fuera de rango", () => {
    assert.throws(
        () => projectSolvency(policy, input({ payoutReserve: "-1" })),
        /must not be negative/,
    );
    assert.throws(
        () => projectSolvency({ ...policy, reserveShockBps: 10_001 }, input()),
        /between 0 and 10000/,
    );
});

test("rechaza inconsistencias de concentracion", () => {
    assert.throws(
        () => projectSolvency(policy, input({ largestRouteLiability: "1101" })),
        /exceeds total obligations/,
    );
});

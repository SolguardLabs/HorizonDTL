"use strict";

const BPS_SCALE = 10_000n;

function projectSolvency(policy, input) {
    const normalizedPolicy = {
        reserveShockBps: toBps(policy.reserveShockBps, "reserveShockBps"),
        sourceRecoveryBps: toBps(policy.sourceRecoveryBps, "sourceRecoveryBps"),
        backstopRecoveryBps: toBps(policy.backstopRecoveryBps, "backstopRecoveryBps"),
        queuedCoverageBps: toBps(policy.queuedCoverageBps, "queuedCoverageBps"),
        targetBufferBps: toBps(policy.targetBufferBps, "targetBufferBps"),
        maxRouteConcentrationBps: toBps(
            policy.maxRouteConcentrationBps,
            "maxRouteConcentrationBps",
        ),
        reserveFloor: toUnits(policy.reserveFloor, "reserveFloor"),
    };
    if (normalizedPolicy.maxRouteConcentrationBps === 0n) {
        throw new RangeError("maxRouteConcentrationBps must be positive");
    }

    const values = Object.fromEntries(
        Object.entries(input).map(([key, value]) => [key, toUnits(value, key)]),
    );
    const queuedLiability = values.queuedLocalLiability + values.queuedRoutedLiability;
    const totalObligations = values.liveRoutedLiability + queuedLiability;
    if (values.largestRouteLiability > totalObligations) {
        throw new RangeError("largestRouteLiability exceeds total obligations");
    }

    const stressedPayoutReserve = mulBpsFloor(
        values.payoutReserve,
        BPS_SCALE - normalizedPolicy.reserveShockBps,
    );
    const eligibleSourceReserve = mulBpsFloor(
        values.sourceReserve,
        normalizedPolicy.sourceRecoveryBps,
    );
    const eligibleBackstop = mulBpsFloor(
        values.backstopCapacity,
        normalizedPolicy.backstopRecoveryBps,
    );
    const eligibleLiquidity = stressedPayoutReserve + eligibleSourceReserve + eligibleBackstop;
    const requiredLiquidity =
        values.liveRoutedLiability +
        mulBpsFloor(queuedLiability, normalizedPolicy.queuedCoverageBps) +
        normalizedPolicy.reserveFloor;
    const targetLiquidity =
        requiredLiquidity + mulBpsFloor(requiredLiquidity, normalizedPolicy.targetBufferBps);
    const capitalShortfall =
        eligibleLiquidity < requiredLiquidity ? requiredLiquidity - eligibleLiquidity : 0n;
    const capitalBuffer =
        eligibleLiquidity > requiredLiquidity ? eligibleLiquidity - requiredLiquidity : 0n;
    const coverageBps = ratioBps(eligibleLiquidity, requiredLiquidity);
    const routeConcentrationBps =
        totalObligations === 0n ? 0n : ratioBps(values.largestRouteLiability, totalObligations);

    let band = "normal";
    if (capitalShortfall > 0n) {
        band = "critical";
    } else if (routeConcentrationBps > normalizedPolicy.maxRouteConcentrationBps) {
        band = "restricted";
    } else if (eligibleLiquidity < targetLiquidity) {
        band = "guarded";
    }

    return freezeProjection({
        band,
        stressedPayoutReserve,
        eligibleSourceReserve,
        eligibleBackstop,
        eligibleLiquidity,
        totalObligations,
        requiredLiquidity,
        targetLiquidity,
        capitalBuffer,
        capitalShortfall,
        coverageBps,
        routeConcentrationBps,
    });
}

function mulBpsFloor(value, bps) {
    return (value * bps) / BPS_SCALE;
}

function ratioBps(numerator, denominator) {
    return denominator === 0n ? BPS_SCALE : (numerator * BPS_SCALE) / denominator;
}

function toUnits(value, field) {
    let units;
    try {
        units = BigInt(value);
    } catch {
        throw new TypeError(`${field} must be an integer-compatible value`);
    }
    if (units < 0n) {
        throw new RangeError(`${field} must not be negative`);
    }
    return units;
}

function toBps(value, field) {
    const bps = toUnits(value, field);
    if (bps > BPS_SCALE) {
        throw new RangeError(`${field} must be between 0 and 10000`);
    }
    return bps;
}

function freezeProjection(projection) {
    return Object.freeze(
        Object.fromEntries(
            Object.entries(projection).map(([key, value]) => [
                key,
                typeof value === "bigint" ? value.toString() : value,
            ]),
        ),
    );
}

module.exports = { projectSolvency };

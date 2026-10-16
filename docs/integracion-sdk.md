# Integración y cliente

## Contrato Rust

El consumidor importa únicamente tipos reexportados por `horizon_dtl`. La construcción de
objetos puede ocurrir fuera del proceso, pero la validación y mutación pertenecen al ledger.

```mermaid
flowchart LR
    APP[Aplicación] --> PUB[src/lib.rs]
    PUB --> LEDGER[HorizonLedger]
    PUB --> SOLV[SolvencyModel]
    PUB --> TYPES[Tipos firmables]
    LEDGER --> RESULT[TxId / HorizonError]
    SOLV --> PROJ[SolvencyProjection]
```

## Cliente JavaScript

`client/horizon.js` implementa la proyección de solvencia con `BigInt`. Acepta cadenas o
enteros compatibles y devuelve un objeto congelado cuyas magnitudes son cadenas decimales.

```mermaid
sequenceDiagram
    participant U as Operador
    participant J as Cliente JS
    participant R as Núcleo Rust
    U->>J: policy + snapshot
    J-->>U: proyección previa
    U->>R: mismo policy + snapshot
    R-->>U: proyección autoritativa
    U->>U: compara campos deterministas
```

No conviertas importes a `Number`; valores superiores a `2^53 - 1` pierden precisión.

## Ejemplo completo

```javascript
const { projectSolvency } = require("../client/horizon");

const projection = projectSolvency(
    {
        reserveShockBps: 750,
        sourceRecoveryBps: 7000,
        backstopRecoveryBps: 4000,
        queuedCoverageBps: 9000,
        targetBufferBps: 1500,
        maxRouteConcentrationBps: 4500,
        reserveFloor: "5000000000",
    },
    {
        payoutReserve: "90000000000",
        sourceReserve: "25000000000",
        liveRoutedLiability: "60000000000",
        queuedLocalLiability: "7000000000",
        queuedRoutedLiability: "8000000000",
        backstopCapacity: "15000000000",
        largestRouteLiability: "22000000000",
    },
);

if (projection.band !== "normal") {
    console.log("review required", projection);
}
```

## Adaptación de errores

```mermaid
flowchart TD
    HE[HorizonError] --> AUTH[Identidad / firma / nonce]
    HE --> POLICY[Política / límites / temporalidad]
    HE --> STATE[Entidad ausente o duplicada]
    HE --> MATH[Aritmética / conservación]
    AUTH --> R4[Rechazo definitivo de solicitud]
    POLICY --> R4
    STATE --> R4
    MATH --> STOP[Detener y revisar estado]
```

No reintentes automáticamente un error determinista con la misma entrada. Un reintento solo
tiene sentido después de obtener nonce, epoch, configuración o snapshot actualizados.

## Paridad

La paridad entre implementaciones se comprueba con vectores que incluyen:

- proyección `normal` con buffer;
- shortfall y banda `critical`;
- concentración y banda `restricted`;
- valores negativos y basis points fuera de rango;
- inconsistencia entre mayor ruta y total de obligaciones;
- magnitudes superiores al rango seguro de JavaScript `Number`.

Rust es la referencia para una transición. El cliente JavaScript sirve para prevalidación,
interfaces operativas y comparación de snapshots.

# Rutas de liquidación

## Topología

Una `SettlementRoute` autoriza un recorrido concreto entre dos vaults del mismo activo. La
existencia de una ruta no sustituye los controles de identidad, temporalidad, reserva o fee.

```mermaid
flowchart LR
    V1[Vault origen A] -->|ruta local| V1
    V1 -->|ruta R1| V2[Vault pagador B]
    V1 -->|ruta R2| V3[Vault pagador C]
    V2 -->|ruta R3| V4[Vault pagador D]
```

## Admisión

```mermaid
sequenceDiagram
    participant L as Ledger
    participant B as RouteBook
    participant O as OracleBook
    participant R as RiskEngine
    participant V as VaultState
    L->>B: resolve(source, payout, asset)
    B-->>L: límites y ventana
    L->>O: latest(payout_vault)
    O-->>L: observación vigente
    L->>R: evaluate_ticket(...)
    R->>V: reserva proyectada
    V-->>R: capacidad
    R-->>L: RiskSnapshot
```

Orden recomendado de validación:

1. correspondencia de red, nota, activo y beneficiario;
2. firma, nonce y no duplicidad;
3. madurez de la nota y epoch del protocolo;
4. existencia de ruta y límites de importe;
5. observación válida para el vault pagador;
6. fee del relayer y reserva proyectada;
7. aplicación contable y journal.

## Estado de una ruta

```mermaid
stateDiagram-v2
    [*] --> Configurada
    Configurada --> Disponible: epoch dentro de ventana
    Disponible --> Disponible: liquidación aceptada
    Disponible --> FueraDeVentana: epoch avanza
    Configurada --> Suspendida: política operativa
    Suspendida --> Configurada: revisión aprobada
    FueraDeVentana --> [*]
```

El libro de rutas es una estructura de política. Las operaciones históricas permanecen en el
journal aunque la configuración posterior cambie.

## Concentración

La proyección de solvencia mide la mayor obligación de ruta frente al total:

```text
route_concentration_bps = floor(largest_route_liability × 10_000 / total_obligations)
```

Un exceso de concentración produce la banda `restricted` incluso cuando la liquidez requerida
está cubierta. Esto separa capacidad agregada de dependencia excesiva de un único corredor.

## Prácticas operativas

- asignar salts distintos a cada ruta;
- evitar mínimos tan bajos que generen fragmentación contable;
- dimensionar máximos con la reserva posterior, no solo con saldo actual;
- revisar concentración y cola en el mismo snapshot;
- registrar cambios de política con epoch efectivo y revisión independiente.

# Arquitectura de HorizonDTL

## Propósito

HorizonDTL separa autorización, política, contabilidad y evidencia. El ledger es el único
punto de mutación; los demás módulos aportan tipos de dominio y validaciones deterministas.

```mermaid
flowchart TB
    API[src/lib.rs] --> L[ledger]
    API --> S[solvency]
    L --> A[accounts]
    L --> N[notes]
    L --> T[tickets]
    L --> V[vault]
    L --> R[routes]
    L --> O[oracle]
    L --> K[risk]
    L --> F[fees]
    L --> J[journal]
    N --> C[codec]
    T --> C
    C --> X[crypto]
    A --> I[ids]
    V --> M[amount]
```

## Capas

| Capa         | Responsabilidad                                             |
| ------------ | ----------------------------------------------------------- |
| Contrato     | tipos exportados y compatibilidad semántica                 |
| Autorización | vista canónica, identidad, firma, nonce y epoch             |
| Política     | roles, rutas, fees, límites y observaciones                 |
| Ejecución    | transición indivisible del ledger                           |
| Contabilidad | cuentas, reservas, notional y fee ledger                    |
| Evidencia    | operación de journal, actor, importes, referencias y digest |
| Proyección   | escenarios de cobertura y concentración sin mutar estado    |

## Frontera de mutación

```mermaid
sequenceDiagram
    participant C as Consumidor
    participant D as Objeto de dominio
    participant L as HorizonLedger
    participant P as Políticas
    participant S as Estado
    participant J as Journal
    C->>D: Construye y firma
    C->>L: Envía objeto firmado
    L->>D: Verifica bytes autorizados
    L->>P: Evalúa roles, tiempo y límites
    P-->>L: Decisión
    L->>S: Aplica débitos y créditos
    L->>L: Comprueba conservación
    L->>J: Añade operación y digest
    L-->>C: TxId
```

La secuencia evita estados intermedios observables. Un error devuelve el estado anterior sin
avanzar nonces ni registrar la operación.

## Propiedad de los datos

```mermaid
erDiagram
    ACCOUNT ||--o{ INGRESS_ORDER : signs
    ACCOUNT ||--o{ REDEMPTION_TICKET : signs
    VAULT ||--o{ DELIVERY_NOTE : issues
    DELIVERY_NOTE ||--o| REDEMPTION_TICKET : settles
    VAULT ||--o{ SETTLEMENT_ROUTE : sources
    VAULT ||--o{ SETTLEMENT_ROUTE : pays
    VAULT ||--o{ INDEX_OBSERVATION : observes
    ASSET ||--o{ VAULT : denominates
    TRANSACTION ||--|| JOURNAL_ENTRY : records
```

Los identificadores son valores derivados y no contadores globales. Esto hace que una misma
entrada produzca la misma identidad lógica y permite detectar duplicados antes de la mutación.

## Serialización y digests

`codec::canonical_bytes` es el formato común para firmas y digests. Los campos se ordenan por
estructura, no por el orden de un mapa externo. Los consumidores deben reutilizar las vistas
de autorización exportadas en vez de reconstruir mensajes de firma manualmente.

## Compatibilidad

La API pública se concentra en `src/lib.rs`. Los módulos internos pueden evolucionar sin
afectar consumidores mientras se conserven tipos, validaciones y semántica documentada. Un
cambio incompatible requiere nueva versión mayor y migración explícita.

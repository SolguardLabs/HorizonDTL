# Politica de Seguridad

Horizon DTL esta disenado como una libreria de ledger para liquidacion
distribuida. La seguridad operacional se apoya en firmas, roles, nonces,
digests canonicos, limites de riesgo y verificacion de conservacion por activo.

## Alcance

Esta politica cubre:

- el crate Rust `horizon_dtl`;
- la API publica expuesta por `src/lib.rs`;
- las transiciones del ledger;
- los modelos de vaults, notas, rutas y tickets;
- los tests Rust y JavaScript;
- scripts de CI y workflows de GitHub Actions.

No cubre despliegues externos, frontends, infraestructura fuera del repositorio
ni integraciones privadas que consuman la libreria.

## Controles de Seguridad

El protocolo implementa los siguientes controles:

- Firmas Ed25519 para ordenes de ingreso y tickets de redencion.
- Nonces independientes por cuenta para emision y redencion.
- Identificadores derivados para cuentas, vaults, notas, tickets y
  transacciones.
- Digests canonicos para serializacion reproducible.
- Roles operativos para emisores, beneficiarios, relayers, oraculos,
  tesoreria y controladores de vault.
- Limites de riesgo sobre importe, fee y reserva posterior.
- Observaciones de indice ligadas a autoridad configurada por vault.
- Journal secuencial con digest de estado.
- Verificacion de conservacion contable por activo.

## Gestion de Dependencias

Las dependencias Rust se fijan con `Cargo.lock` y las dependencias JavaScript
con `bun.lock`. Dependabot revisa semanalmente:

- crates de Cargo;
- paquetes gestionados por Bun;
- acciones de GitHub.

Toda actualizacion de dependencias debe pasar el pipeline completo antes de ser
integrada.

## Validacion Requerida

Antes de aceptar cambios se debe ejecutar:

```bash
bun run ci
```

Para validar solo tests:

```bash
bun run test:all
```

Los cambios sobre transiciones economicas, autorizacion, firmas, rutas,
vaults, fees u oracle deben incluir pruebas de regresion cuando modifiquen el
comportamiento observable.

## Comunicacion de Incidencias

Los hallazgos sensibles deben comunicarse de forma privada al equipo mantenedor.
Un reporte util debe incluir:

- version o commit analizado;
- descripcion del comportamiento observado;
- pasos de reproduccion;
- impacto sobre saldos, autorizacion, rutas o estado;
- salida relevante de comandos, tests o trazas.

No se deben publicar detalles tecnicos reproducibles en issues publicos antes de
una revision coordinada.

## Criterios de Aceptacion

Un cambio se considera listo cuando:

- mantiene `cargo fmt` y `clippy` sin advertencias;
- conserva determinismo de digests;
- no rompe la conservacion por activo;
- mantiene verdes los tests Rust y Bun;
- actualiza pruebas cuando cambia un contrato observable;
- no reduce controles existentes sin justificacion tecnica.

# Contribuir a HorizonDTL

## Flujo

1. Crea una rama desde `main`.
2. Mantén cada cambio limitado a un dominio.
3. Añade pruebas para flujos aceptados y rechazados.
4. Ejecuta `bun run ci`.
5. Describe invariantes afectadas, compatibilidad y evidencia de validación.

## Criterios técnicos

- Código y comentarios de código en inglés.
- Documentación pública en español.
- Aritmética monetaria sin coma flotante.
- Ninguna mutación antes de completar la validación.
- Errores deterministas y sin material sensible.
- Cambios de API acompañados de documentación y ejemplo.
- Dependencias nuevas justificadas por una capacidad concreta.

## Commits

Usa mensajes imperativos y un propósito por commit. No mezcles refactors amplios con cambios
de comportamiento. Antes de solicitar revisión, confirma que el árbol está limpio y que los
lockfiles corresponden a los manifiestos.

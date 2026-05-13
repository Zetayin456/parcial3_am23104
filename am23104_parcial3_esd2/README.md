# Motor de Catálogo AVL en Rust

Sistema de gestión de inventario de libros usando un **Árbol AVL** implementado en Rust estable, sin dependencias externas.

---

## ¿Qué es un Árbol AVL?

Un árbol AVL es un árbol binario de búsqueda (BST) **autobalanceado**. Tras cada inserción o eliminación, verifica el *factor de balance* de cada nodo (diferencia de alturas entre subárbol izquierdo y derecho). Si ese factor supera ±1, aplica **rotaciones** para corregirlo.

Esto garantiza que las operaciones de búsqueda, inserción y eliminación siempre sean **O(log n)**, sin importar el orden de inserción.

---

## Estructura del Proyecto

```
catalogo_avl/
├── Cargo.toml
└── src/
    └── main.rs
```

---

## Cómo Ejecutar

```bash
# Compilar
cargo build

# Ejecutar
cargo run

# Compilar en modo release (optimizado)
cargo build --release
```

---

## Funcionalidades Implementadas

| Fase | Función | Descripción |
|------|---------|-------------|
| Base | `insertar` | Inserta un libro y rebalancea el árbol |
| Base | `imprimir` | Muestra el árbol rotado 90° en consola |
| 1 | Comentarios técnicos | Documentación de `Box`, `Option`, `take()`, rotaciones |
| 2 | `buscar` | Busca un libro por ISBN sin copias (retorna `&Libro`) |
| 3 | `eliminar` | Elimina un nodo (hoja, un hijo, dos hijos) con rebalanceo |
| 4B | `calcular_estadisticas` | Retorna altura, total de nodos y libro con ISBN máximo |

---

## Salida Esperada al Ejecutar

```
==============================================
  Motor de Catálogo AVL - Biblioteca Municipal
==============================================

>>> Insertando libros en el árbol AVL...
    + ISBN  10: El Quijote
    + ISBN  20: 1984
    + ISBN  30: Hamlet
    + ISBN   5: Fahrenheit 451
    + ISBN   2: La Odisea
    + ISBN  25: El Principito

>>> Árbol AVL actual (rotado 90°, raíz a la izquierda):

    [ISBN: 30] Hamlet
        [ISBN: 25] El Principito
[ISBN: 20] 1984
        [ISBN: 10] El Quijote
    [ISBN: 5] Fahrenheit 451
        [ISBN: 2] La Odisea

==============================================
  FASE 2: Búsqueda
==============================================
✓ Encontrado ISBN 20: "1984"
✗ ISBN 99 no encontrado (esperado).
✗ Búsqueda en árbol vacío: None (correcto).

==============================================
  FASE 3: Eliminación
==============================================
[... árbol tras cada eliminación ...]

==============================================
  FASE 4: Estadísticas del árbol
==============================================
  Altura total del árbol : 3
  Total de nodos         : 6
  Libro con ISBN máximo  : [30] "Hamlet"
```

---

## Árbol Final Tras Insertar [10, 20, 30, 5, 2, 25]

```
         20  ← raíz (balanceada)
        /  \
       5    25
      / \   / \
     2  10 -  30
```

### Rotaciones que ocurrieron:

1. **Después de insertar 30** (secuencia 10→20→30):
   - Balance en 10 = -2 (cargado a la derecha, caso DD)
   - **Rotación simple a la izquierda** sobre el nodo 10
   - Resultado: 20 sube a raíz

2. **Después de insertar 2** (secuencia 5→2):
   - Al verificar en el nodo 20: balance = +2 (cargado a la izq)
   - El hijo izquierdo (5) tiene balance +1 → caso II
   - **Rotación simple a la derecha** sobre el nodo 20
   - El árbol se rebalancea correctamente

---

## Por Qué `.take()` es Necesario en las Rotaciones

`take()` extrae el valor de un `Option<T>` dejando `None` en su lugar. Es necesario porque:

1. Rust prohíbe mover un campo fuera de una estructura mientras la estructura pueda seguir siendo usada.
2. En `rotar_derecha`, necesitamos mover `y.izquierdo` (que es un `Box<Nodo>`) para convertirlo en la nueva raíz.
3. Una asignación directa (`let x = y.izquierdo`) haría un *partial move*: `y` quedaría parcialmente movido y el compilador lo rechaza porque no puede garantizar que `y` sea válido después.
4. `take()` resuelve esto: deja `y.izquierdo = None` (estado válido) y nos entrega el `Box<Nodo>` extraído.
5. Sin `take()`, tendríamos que clonar los nodos, lo cual es ineficiente y va en contra del diseño de Rust.

---

## Decisiones de Diseño

- **Sin `.clone()` innecesarios**: la búsqueda devuelve `&Libro` (referencia), no una copia.
- **Lifetimes explícitos en `buscar` y `calcular_estadisticas`**: para dejar claro al compilador que la referencia devuelta vive mientras viva el árbol.
- **`extraer_minimo` usa `mut nodo`**: permite usar `take()` sobre los campos sin partial moves.
- **Código en español**: siguiendo las convenciones del enunciado.

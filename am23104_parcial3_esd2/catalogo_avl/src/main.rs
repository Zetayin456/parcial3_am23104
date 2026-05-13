// ============================================================
// Motor de Catálogo AVL en Rust
// Materia: Estructuras de Datos / Programación II
// ============================================================
//
// CONCEPTOS CLAVE DE RUST USADOS AQUÍ:
//
// Box<T>: Es un puntero que guarda datos en el heap (memoria dinámica).
//   Se usa porque los nodos del árbol son recursivos: un Nodo contiene
//   otros Nodos, y Rust necesita saber el tamaño exacto de cada tipo en
//   tiempo de compilación. Box rompe esa recursión infinita de tamaño.
//
// Option<T>: Representa un valor que puede existir (Some) o no (None).
//   Aquí se usa para los hijos de cada nodo: un nodo puede tener hijo
//   izquierdo/derecho o no tenerlo.
//
// take(): Método de Option<T> que extrae el valor interno dejando None
//   en su lugar. Es esencial en las rotaciones porque Rust no permite
//   mover un valor que está "prestado" dentro de una estructura. take()
//   es la solución idiomática: mueve el valor de forma segura y deja
//   la referencia original en estado válido (None).
//
// as_ref(): Convierte &Option<T> en Option<&T>. Permite acceder al
//   contenido de un Option sin consumirlo ni clonarlo.
//
// Ownership y Borrowing: Rust garantiza que cada valor tiene un único
//   dueño. Las funciones que reciben Box<Nodo> por valor son las dueñas
//   del nodo. Las que reciben &Option<Box<Nodo>> solo lo prestan.

// ---------------------------------------------------------------
// ESTRUCTURAS DE DATOS
// ---------------------------------------------------------------

#[derive(Debug, Clone)]
struct Libro {
    isbn: u32,
    titulo: String,
}

struct Nodo {
    libro: Libro,
    // Option<Box<Nodo>>: el hijo puede existir o no (Option),
    // y si existe, vive en el heap (Box).
    izquierdo: Option<Box<Nodo>>,
    derecho: Option<Box<Nodo>>,
    altura: i32,
}

impl Nodo {
    fn nuevo(libro: Libro) -> Self {
        Nodo {
            libro,
            izquierdo: None,
            derecho: None,
            altura: 1, // todo nodo hoja empieza con altura 1
        }
    }
}

// ---------------------------------------------------------------
// FUNCIONES AUXILIARES DE ALTURA Y BALANCE
// ---------------------------------------------------------------

// Recibe una referencia al Option para no consumirlo.
// map_or devuelve 0 si es None, o la altura del nodo si es Some.
fn obtener_altura(nodo: &Option<Box<Nodo>>) -> i32 {
    nodo.as_ref().map_or(0, |n| n.altura)
}

fn actualizar_altura(nodo: &mut Nodo) {
    nodo.altura = 1 + std::cmp::max(
        obtener_altura(&nodo.izquierdo),
        obtener_altura(&nodo.derecho),
    );
}

// Balance = altura_izquierda - altura_derecha
// >1 => árbol cargado a la izquierda
// <-1 => árbol cargado a la derecha
fn obtener_balance(nodo: &Nodo) -> i32 {
    obtener_altura(&nodo.izquierdo) - obtener_altura(&nodo.derecho)
}

// ---------------------------------------------------------------
// ROTACIONES AVL
//
// Las rotaciones reorganizan los nodos para mantener el árbol balanceado
// sin perder el orden BST. Hay 4 casos:
//   - Izquierda-Izquierda (II): rotación simple a la derecha
//   - Derecha-Derecha (DD): rotación simple a la izquierda
//   - Izquierda-Derecha (ID): rotación doble (izq en hijo, der en raíz)
//   - Derecha-Izquierda (DI): rotación doble (der en hijo, izq en raíz)
//
// POR QUÉ .take() EN LAS ROTACIONES:
//   En rotar_derecha recibimos 'y' por valor (somos dueños).
//   Para extraer y.izquierdo y reasignarlo, necesitamos mover ese Box
//   fuera de 'y'. Rust prohíbe mover campos de una estructura cuando
//   existe alguna referencia activa a la estructura, aunque aquí no haya
//   referencias externas: el compilador no puede verificarlo campo a campo.
//   .take() es la solución: mueve el valor del Option, dejando None, lo
//   que satisface al borrow checker sin necesidad de clonar nada.
// ---------------------------------------------------------------

// Rotación simple a la derecha (caso Izquierda-Izquierda)
//
//      y                  x
//     / \                / \
//    x   T3    =>      T1   y
//   / \                    / \
//  T1  T2                T2  T3
fn rotar_derecha(mut y: Box<Nodo>) -> Box<Nodo> {
    // take() extrae el hijo izquierdo de y, dejando None en y.izquierdo
    let mut x = y.izquierdo.take().expect("Hijo izquierdo ausente");

    // T2 (hijo derecho de x) pasa a ser hijo izquierdo de y
    y.izquierdo = x.derecho.take();

    actualizar_altura(&mut y);

    // y pasa a ser hijo derecho de x
    x.derecho = Some(y);

    actualizar_altura(&mut x);

    x // x es la nueva raíz del subárbol
}

// Rotación simple a la izquierda (caso Derecha-Derecha)
//
//    x                    y
//   / \                  / \
//  T1   y      =>       x   T3
//      / \             / \
//     T2  T3          T1  T2
fn rotar_izquierda(mut x: Box<Nodo>) -> Box<Nodo> {
    let mut y = x.derecho.take().expect("Hijo derecho ausente");

    // T2 pasa a ser hijo derecho de x
    x.derecho = y.izquierdo.take();

    actualizar_altura(&mut x);

    y.izquierdo = Some(x);

    actualizar_altura(&mut y);

    y
}

// ---------------------------------------------------------------
// INSERCIÓN CON REBALANCEO
// ---------------------------------------------------------------

fn insertar(nodo_opt: Option<Box<Nodo>>, libro: Libro) -> Box<Nodo> {
    // Si llegamos a un lugar vacío, creamos el nuevo nodo aquí
    let mut nodo = match nodo_opt {
        None => return Box::new(Nodo::nuevo(libro)),
        Some(n) => n,
    };

    let isbn_nuevo = libro.isbn;

    // Insertamos recursivamente según el orden BST
    if isbn_nuevo < nodo.libro.isbn {
        // take() mueve el hijo izquierdo para pasarlo a insertar()
        nodo.izquierdo = Some(insertar(nodo.izquierdo.take(), libro));
    } else if isbn_nuevo > nodo.libro.isbn {
        nodo.derecho = Some(insertar(nodo.derecho.take(), libro));
    } else {
        // ISBN duplicado: no insertamos
        return nodo;
    }

    actualizar_altura(&mut nodo);

    let balance = obtener_balance(&nodo);

    // Caso II: desbalance izquierdo, hijo izquierdo también cargado a la izquierda
    if balance > 1 && isbn_nuevo < nodo.izquierdo.as_ref().unwrap().libro.isbn {
        return rotar_derecha(nodo);
    }

    // Caso DD: desbalance derecho, hijo derecho también cargado a la derecha
    if balance < -1 && isbn_nuevo > nodo.derecho.as_ref().unwrap().libro.isbn {
        return rotar_izquierda(nodo);
    }

    // Caso ID: desbalance izquierdo, pero inserción fue en subárbol derecho del hijo
    if balance > 1 && isbn_nuevo > nodo.izquierdo.as_ref().unwrap().libro.isbn {
        let hijo_izq = nodo.izquierdo.take().unwrap();
        nodo.izquierdo = Some(rotar_izquierda(hijo_izq)); // primero rotamos el hijo
        return rotar_derecha(nodo);                        // luego la raíz
    }

    // Caso DI: desbalance derecho, pero inserción fue en subárbol izquierdo del hijo
    if balance < -1 && isbn_nuevo < nodo.derecho.as_ref().unwrap().libro.isbn {
        let hijo_der = nodo.derecho.take().unwrap();
        nodo.derecho = Some(rotar_derecha(hijo_der));
        return rotar_izquierda(nodo);
    }

    nodo
}

// ---------------------------------------------------------------
// BÚSQUEDA (FASE 2)
//
// Recibe una referencia, devuelve una referencia: sin clones.
// El lifetime queda implícito: la referencia devuelta vive mientras
// viva el árbol (el compilador lo infiere correctamente).
// ---------------------------------------------------------------

fn buscar<'a>(nodo: &'a Option<Box<Nodo>>, isbn: u32) -> Option<&'a Libro> {
    // if let extrae la referencia al nodo sin consumir el Option
    if let Some(n) = nodo {
        if isbn == n.libro.isbn {
            Some(&n.libro) // devolvemos referencia al libro encontrado
        } else if isbn < n.libro.isbn {
            buscar(&n.izquierdo, isbn) // buscamos en subárbol izquierdo
        } else {
            buscar(&n.derecho, isbn)   // buscamos en subárbol derecho
        }
    } else {
        None // llegamos a un lugar vacío: no existe
    }
}

// ---------------------------------------------------------------
// ELIMINACIÓN CON REBALANCEO (FASE 3)
//
// Tres casos:
//   1. Nodo hoja: simplemente lo eliminamos (devolvemos None)
//   2. Nodo con un hijo: lo reemplazamos con ese hijo
//   3. Nodo con dos hijos: lo reemplazamos con su sucesor in-order
//      (el nodo más a la izquierda del subárbol derecho), luego
//      eliminamos el sucesor de su posición original.
// ---------------------------------------------------------------

// Función auxiliar: extrae el nodo con el ISBN mínimo de un subárbol.
// Devuelve (nodo_mínimo, subárbol_sin_ese_nodo).
fn extraer_minimo(mut nodo: Box<Nodo>) -> (Box<Nodo>, Option<Box<Nodo>>) {
    if nodo.izquierdo.is_none() {
        // Este es el mínimo. Extraemos su hijo derecho (que puede existir o no)
        // y devolvemos el nodo limpio junto al subárbol restante.
        let derecho = nodo.derecho.take();
        (nodo, derecho)
    } else {
        // Hay un nodo más pequeño a la izquierda, seguimos bajando
        // Necesitamos tomar el nodo como mutable para poder hacer take()
        let mut nodo = nodo;
        let izquierdo = nodo.izquierdo.take().unwrap();
        let (minimo, nuevo_izq) = extraer_minimo(izquierdo);
        nodo.izquierdo = nuevo_izq;
        actualizar_altura(&mut nodo);
        (minimo, Some(nodo))
    }
}

fn eliminar(nodo_opt: Option<Box<Nodo>>, isbn: u32) -> Option<Box<Nodo>> {
    let mut nodo = match nodo_opt {
        None => return None, // ISBN no encontrado
        Some(n) => n,
    };

    if isbn < nodo.libro.isbn {
        // El nodo a eliminar está en el subárbol izquierdo
        nodo.izquierdo = eliminar(nodo.izquierdo.take(), isbn);
    } else if isbn > nodo.libro.isbn {
        // El nodo a eliminar está en el subárbol derecho
        nodo.derecho = eliminar(nodo.derecho.take(), isbn);
    } else {
        // Encontramos el nodo a eliminar
        match (nodo.izquierdo.take(), nodo.derecho.take()) {
            // Caso 1: nodo hoja (sin hijos)
            (None, None) => return None,

            // Caso 2a: solo tiene hijo derecho
            (None, Some(der)) => return Some(der),

            // Caso 2b: solo tiene hijo izquierdo
            (Some(izq), None) => return Some(izq),

            // Caso 3: tiene dos hijos -> usamos sucesor in-order
            (Some(izq), Some(der)) => {
                // El sucesor es el mínimo del subárbol derecho
                let (sucesor, nuevo_der) = extraer_minimo(der);

                // Construimos el nodo sucesor con los hijos del nodo eliminado
                let mut nuevo_nodo = Box::new(Nodo {
                    libro: sucesor.libro,
                    izquierdo: Some(izq),
                    derecho: nuevo_der,
                    altura: 1,
                });
                actualizar_altura(&mut nuevo_nodo);
                nodo = nuevo_nodo;
            }
        }
    }

    // Actualizamos altura y rebalanceamos (igual que en insertar)
    actualizar_altura(&mut nodo);
    let balance = obtener_balance(&nodo);

    // Caso II
    if balance > 1 && obtener_balance(nodo.izquierdo.as_ref().unwrap()) >= 0 {
        return Some(rotar_derecha(nodo));
    }
    // Caso ID
    if balance > 1 && obtener_balance(nodo.izquierdo.as_ref().unwrap()) < 0 {
        let hijo_izq = nodo.izquierdo.take().unwrap();
        nodo.izquierdo = Some(rotar_izquierda(hijo_izq));
        return Some(rotar_derecha(nodo));
    }
    // Caso DD
    if balance < -1 && obtener_balance(nodo.derecho.as_ref().unwrap()) <= 0 {
        return Some(rotar_izquierda(nodo));
    }
    // Caso DI
    if balance < -1 && obtener_balance(nodo.derecho.as_ref().unwrap()) > 0 {
        let hijo_der = nodo.derecho.take().unwrap();
        nodo.derecho = Some(rotar_derecha(hijo_der));
        return Some(rotar_izquierda(nodo));
    }

    Some(nodo)
}

// ---------------------------------------------------------------
// ESTADÍSTICAS (FASE 4 - OPCIÓN B)
// ---------------------------------------------------------------

struct Estadisticas<'a> {
    altura: i32,
    total_nodos: u32,
    libro_isbn_maximo: Option<&'a Libro>,
}

fn calcular_estadisticas<'a>(nodo: &'a Option<Box<Nodo>>) -> Estadisticas<'a> {
    match nodo {
        None => Estadisticas {
            altura: 0,
            total_nodos: 0,
            libro_isbn_maximo: None,
        },
        Some(n) => {
            let stats_izq = calcular_estadisticas(&n.izquierdo);
            let stats_der = calcular_estadisticas(&n.derecho);

            // La altura de este subárbol es la del nodo actual
            let altura = n.altura;

            // Total de nodos: yo + izquierda + derecha
            let total_nodos = 1 + stats_izq.total_nodos + stats_der.total_nodos;

            // El libro con ISBN máximo está en el nodo más a la derecha
            // (propiedad BST: el máximo siempre está en el extremo derecho)
            let libro_isbn_maximo = stats_der.libro_isbn_maximo
                .or(Some(&n.libro))
                .or(stats_izq.libro_isbn_maximo);

            Estadisticas { altura, total_nodos, libro_isbn_maximo }
        }
    }
}

// ---------------------------------------------------------------
// IMPRESIÓN DEL ÁRBOL (rotada 90°, raíz a la izquierda)
// ---------------------------------------------------------------

fn imprimir(nodo: &Option<Box<Nodo>>, nivel: usize) {
    if let Some(n) = nodo {
        imprimir(&n.derecho, nivel + 1);
        println!(
            "{:indent$}[ISBN: {}] {}",
            "",
            n.libro.isbn,
            n.libro.titulo,
            indent = nivel * 4
        );
        imprimir(&n.izquierdo, nivel + 1);
    }
}

// ---------------------------------------------------------------
// MAIN: Pruebas de todas las funcionalidades
// ---------------------------------------------------------------

fn main() {
    println!("==============================================");
    println!("  Motor de Catálogo AVL - Biblioteca Municipal");
    println!("==============================================\n");

    // --- INSERCIÓN ---
    let mut raiz: Option<Box<Nodo>> = None;

    let datos = vec![
        (10, "El Quijote"),
        (20, "1984"),
        (30, "Hamlet"),
        (5,  "Fahrenheit 451"),
        (2,  "La Odisea"),
        (25, "El Principito"),
    ];

    println!(">>> Insertando libros en el árbol AVL...");
    for (isbn, titulo) in datos {
        let libro = Libro { isbn, titulo: titulo.to_string() };
        println!("    + ISBN {:>3}: {}", isbn, titulo);
        raiz = Some(insertar(raiz.take(), libro));
    }

    println!("\n>>> Árbol AVL actual (rotado 90°, raíz a la izquierda):\n");
    imprimir(&raiz, 0);

    // --- BÚSQUEDA (FASE 2) ---
    println!("\n==============================================");
    println!("  FASE 2: Búsqueda");
    println!("==============================================");

    let isbn_buscar = 20;
    match buscar(&raiz, isbn_buscar) {
        Some(libro) => println!("✓ Encontrado ISBN {}: \"{}\"", libro.isbn, libro.titulo),
        None        => println!("✗ ISBN {} no encontrado.", isbn_buscar),
    }

    let isbn_inexistente = 99;
    match buscar(&raiz, isbn_inexistente) {
        Some(libro) => println!("✓ Encontrado ISBN {}: \"{}\"", libro.isbn, libro.titulo),
        None        => println!("✗ ISBN {} no encontrado (esperado).", isbn_inexistente),
    }

    // Caso borde: búsqueda en árbol vacío
    let arbol_vacio: Option<Box<Nodo>> = None;
    match buscar(&arbol_vacio, 5) {
        Some(_) => println!("✓ Encontrado (inesperado)"),
        None    => println!("✗ Búsqueda en árbol vacío: None (correcto)."),
    }

    // --- ELIMINACIÓN (FASE 3) ---
    println!("\n==============================================");
    println!("  FASE 3: Eliminación");
    println!("==============================================");

    // Eliminar nodo hoja (ISBN 2 - La Odisea)
    println!("\n>>> Eliminando ISBN 2 (nodo hoja)...");
    raiz = eliminar(raiz.take(), 2);
    imprimir(&raiz, 0);

    // Eliminar nodo con un hijo (ISBN 5 - Fahrenheit 451)
    println!("\n>>> Eliminando ISBN 5 (nodo con posible un hijo)...");
    raiz = eliminar(raiz.take(), 5);
    imprimir(&raiz, 0);

    // Eliminar nodo con dos hijos (ISBN 20 - 1984, raíz del árbol)
    println!("\n>>> Eliminando ISBN 20 (nodo con dos hijos / raíz actual)...");
    raiz = eliminar(raiz.take(), 20);
    imprimir(&raiz, 0);

    // Intentar eliminar ISBN inexistente
    println!("\n>>> Intentando eliminar ISBN 99 (no existe)...");
    raiz = eliminar(raiz.take(), 99);
    println!("    Árbol sin cambios:");
    imprimir(&raiz, 0);

    // --- ESTADÍSTICAS (FASE 4 - OPCIÓN B) ---
    println!("\n==============================================");
    println!("  FASE 4: Estadísticas del árbol");
    println!("==============================================");

    // Reinsertamos todos para tener el árbol completo
    let mut raiz2: Option<Box<Nodo>> = None;
    for (isbn, titulo) in [(10u32, "El Quijote"), (20, "1984"), (30, "Hamlet"),
                            (5, "Fahrenheit 451"), (2, "La Odisea"), (25, "El Principito")] {
        let libro = Libro { isbn, titulo: titulo.to_string() };
        raiz2 = Some(insertar(raiz2.take(), libro));
    }

    let stats = calcular_estadisticas(&raiz2);
    println!("  Altura total del árbol : {}", stats.altura);
    println!("  Total de nodos         : {}", stats.total_nodos);
    match stats.libro_isbn_maximo {
        Some(libro) => println!("  Libro con ISBN máximo  : [{}] \"{}\"", libro.isbn, libro.titulo),
        None        => println!("  Árbol vacío."),
    }

    println!("\n==============================================");
    println!("  Fin del programa.");
    println!("==============================================");
}

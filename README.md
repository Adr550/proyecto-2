# Diorama del Cretácico

Bioma de cubos texturizados mediante **ray tracing en CPU, escrito en Rust sin dependencias de Cargo**. Es la evolución del proyecto original `bosque-carbonifero`, basada en los principios del proyecto `esfera`. Abre directamente el diorama, sin planeta, navegador ni servidor.

## Ejecutar en macOS

```sh
cd /Users/luisestrada/Desktop/proyecto-2
cargo run --release --offline
```

Requiere Rust y las herramientas de desarrollo de macOS. La vista incluye controles discretos de zoom y meteorito en la parte inferior. Las fichas aparecen al hacer clic en animales o plantas, y se cierran con C, Esc o la X. La ventana usa FFI directo a AppKit, CoreGraphics, QuartzCore y el runtime Objective-C que proporciona macOS; no se descarga ni enlaza ningún framework de terceros. El audio usa `/usr/bin/afplay`, también del sistema. No usa raylib, SDL, winit, OpenGL, JavaScript ni paquetes externos.

## Explorar

- **Arrastrar:** orbitar alrededor del centro de interés.
- **Rueda, teclas +/− o botones −/+ inferiores:** zoom proporcional a la distancia.
- **WASD o flechas:** rotar la cámara. A/D giran a los lados y W/S cambian la inclinación. Mantén pulsadas las teclas para un giro continuo.
- **1:** bioma completo. **2:** Triceratops. **3:** Tyrannosaurus. **4:** manada de Edmontosaurus. **5:** Ankylosaurus. **6:** enantiornitas. **7:** anfibios. **8:** montañas. **9:** costa. **0:** pozas costeras con invertebrados.
- **Clic sobre un animal:** acercamiento suave al individuo y ficha. **X:** volver a la cámara anterior. El aspa de la ficha también regresa; C solamente cierra el texto. Las plantas muestran identificación.
- **M:** activar/pausar ambiente. **R:** restaurar vista. **C:** cerrar ficha. **Esc:** cerrar ficha o salir si no hay ficha.

El MP3 compartido está integrado en el ejecutable. Se extrae a un archivo temporal solo al activar sonido, se reproduce en bucle y se elimina al salir normalmente. Es ambientación artística; el título original del audio alude al Jurásico y no demuestra cómo sonaba Hell Creek.

## Bioma y fauna

Llanura fluvial inspirada en Hell Creek, oeste de Norteamérica, aproximadamente **66 millones de años**, antes de la extinción K–Pg. Terreno terrestre de 121 × 97 unidades, ampliado con una costa y mar hasta un total de aproximadamente 121 × 143 unidades con elevaciones de hasta aproximadamente 30 unidades, río sinuoso, laguna, claros, bosque de angiospermas y coníferas, helechos, cantos y madera caída. La fauna comprende:

- **Tyrannosaurus rex**.
- **Triceratops prorsus**, la morfoespecie de la parte superior de Hell Creek.
- **Edmontosaurus annectens**, representado en un grupo de tamaños distintos.
- **Ankylosaurus magniventris**.
- **Enantiornithes indet.**, seis aves arcaicas distribuidas en diferentes zonas, sin asignación inventada a una especie concreta.
- **Scapherpeton tectum**, cuatro anfibios próximos a las riberas y a la laguna.

La documentación y el grado de certeza están en [PALEONTOLOGIA.md](PALEONTOLOGIA.md). Los colores, agrupaciones, posiciones, plumaje y formas vegetales son reconstrucciones estilizadas. No se pretende identificar individuos muertos el día del impacto.

## Implementación

- Dos muestras por píxel en la vista final y cámara perspectiva; framebuffer RGBA propio.
- Intersecciones rayo–AABB por el método de slabs, normales de caras y selección con el mismo algoritmo.
- Cubos para terreno y modelos de animales voxelizados; algunas piezas de agua, capas y follaje son cuboides.
- BVH construida en Rust: inversas precalculadas por rayo, distancias de entrada almacenadas y salida temprana para sombras.
- Luz difusa, brillo especular en agua, reflexión del cielo y geometría mediante rayos secundarios, refracción por Snell, Fresnel y reflexión interna total, sombras por rayos, niebla y cinco materiales procedurales compartidos por todo el bioma.
- Render en segundo plano con `std::thread`, sin bloquear los eventos de la ventana. Vista previa bilineal de resolución adaptativa al arrastrar, usar WASD o hacer zoom; calidad completa con dos muestras tras 180 ms de reposo. Los renders finales obsoletos se cancelan. Abrir/cerrar una ficha reutiliza la imagen ya calculada. No se recalcula la escena en reposo.
- Fuente bitmap propia; no requiere fuentes ni texturas descargadas. Los materiales usan ruido interpolado continuo, con filtrado bilineal de la vista previa y presentación lineal.
- Ventana de 1100 × 756, render final de 1100 × 756; la resolución durante el movimiento se adapta a la CPU. La velocidad depende de CPU; no es un motor GPU ni un path tracer físicamente completo.

## Render sin ventana y comprobaciones

```sh
cargo test --offline
cargo run --release --offline -- --benchmark
cargo run --release --offline -- --render hell-creek.ppm
cargo run --release --offline -- --render triceratops.ppm --view 2
cargo run --release --offline -- --frames 3
```

El render PPM sin ventana también funciona fuera de macOS. `--frames` comprueba el arranque nativo y cierra después de N cuadros. `--help` muestra controles y opciones.

El render original del Carbonífero se conserva sin cambios funcionales en `src/carboniferous.rs` y sigue disponible con:

```sh
cargo run --release --offline --bin carbonifero-original
```

## Meteorito

El botón **Lanzar meteorito** lleva la cámara a la vista general e inicia una secuencia artística de 18 segundos: descenso (0–3 s), destello y onda expansiva (3–6 s), bosque incendiado, carbonización y ceniza. Al terminar quedan troncos sin follaje, una paleta gris y pequeños mamíferos y aves en el suelo. **Ver supervivientes** acerca la cámara a un grupo; **Reiniciar bioma** o R restaura el bosque. El ambiente sonoro del bosque se pausa al lanzar el meteorito.

Las aves finales representan linajes de aves modernas; no son las enantiornitas anteriores. Los mamíferos son multituberculados indeterminados. No se atribuye supervivencia a especies concretas, ni a animales situados en el punto del impacto. La escena final representa un momento posterior y comprime procesos de distintas duraciones. Las llamas y la onda son efectos visuales, no una simulación física de Chicxulub.

Las animaciones usan el render de vista previa en segundo plano y recuperan la calidad completa al terminar. La geometría posterior se prepara una vez, sin reconstruir la BVH en cada cuadro.

```sh
cargo run --release --offline -- --render impacto.ppm --impact 8
cargo run --release --offline -- --render ceniza.ppm --impact 18
cargo run --release --offline -- --render supervivientes.ppm --impact 18 --survivors
```

## Costa y fauna marina

El botón **Costa [9]** y la tecla **9** muestran la playa y el mar; **0** acerca las pozas con cangrejos herradura y moluscos. El agua refracta la imagen y el rayo de selección de los animales sumergidos, manteniendo el zoom y la ficha; **X** regresa a la vista anterior.

- Mosasaurio indeterminado: la ficha indica **reptil marino, no dinosaurio**.
- Plesiosaurio elasmosáurido indeterminado, de cuello largo y cuatro aletas, representativo del Cretácico final.
- Ammonitas, cefalópodos de concha enrollada.
- Cangrejos herradura, bivalvos y nautiloideos indeterminados.

En la etapa posterior al impacto desaparecen mosasaurios, plesiosaurios y ammonitas. Se conservan los individuos representativos de linajes de cangrejos herradura, bivalvos y nautiloideos supervivientes; esto no significa que todas sus especies sobrevivieran. Las ammonitas también son moluscos. Las conchas bivalvas no representan rudistas, que se extinguieron.

La costa es una extensión artística contemporánea del diorama y no una reconstrucción exacta de una localidad de Hell Creek. La superficie marina se calcula analíticamente sin una capa opaca que impida ver o seleccionar la fauna. El fondo y los animales mantienen la geometría de cubos; no hay nuevas dependencias.

```sh
cargo run --release --offline -- --render costa.ppm --view 9
cargo run --release --offline -- --render mosasaurio.ppm --view 10 --info
cargo run --release --offline -- --render plesiosaurio.ppm --view 11
cargo run --release --offline -- --render costa-posterior.ppm --view 9 --impact 18
```

## Cinco materiales y óptica

El diorama activo usa exactamente **cinco materiales**, definidos en `src/material.rs`. Los colores por objeto son tintes del mismo material, no materiales adicionales. La ceniza, el fuego, la atmósfera y la estela del meteorito son efectos sobre la escena y no amplían el registro. El ejecutable histórico `carbonifero-original` es independiente del diorama activo.

| Material | Textura procedural | Albedo RGB | Specular | Exponente | Transparencia | Reflectividad normal | IOR |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Terreno/roca | Estratos y grano | 0.92, 0.88, 0.78 | 0.06 | 12 | 0 | 0 | 1 |
| Madera | Vetas | 0.88, 0.76, 0.60 | 0.10 | 24 | 0 | 0 | 1 |
| Orgánico | Células y moteado | 0.90, 0.96, 0.86 | 0.18 | 38 | 0 | 0 | 1 |
| Agua | Ondas | 0.025, 0.22, 0.33 | 0.70 | 100 | 0.88 | 0.07 | 1.333 |
| Cristal mineral | Inclusiones y bandas | 0.78, 0.92, 0.98 | 0.92 | 160 | 0.95 | 0.045 | 1.52 |

**G** muestra los cristales de la costa; **V** muestra el océano desde un ángulo bajo para observar los reflejos. También se pueden seleccionar los cristales para acercarse y consultar su ficha. **X** regresa. Las vistas de exportación son **13** y **14**, respectivamente.

El océano refleja geometría real y el cielo, y transmite rayos refractados. El clic bajo el agua usa la misma ley de Snell que el render. Los cristales son volúmenes cerrados: los rayos cambian de dirección en la entrada y en la salida; se contempla la reflexión interna total. Las sombras de los cristales transmiten luz. Los parámetros son elecciones visuales, no identificación de una especie mineral concreta.

La textura modula el albedo. `specular` y `shininess` controlan el brillo; `transparency` y `reflectivity` distribuyen el color entre iluminación local y rayos secundarios. La reflectividad aumenta en ángulos rasantes mediante Schlick. Se limitan a cuatro niveles de rayos secundarios en calidad final y dos durante el movimiento. Es un trazador Whitted aproximado: no simula dispersión espectral, cáusticas ni medios dieléctricos anidados arbitrarios.

```sh
cargo run --release --offline -- --render minerales.ppm --view 13 --info
cargo run --release --offline -- --render reflejos.ppm --view 14
```

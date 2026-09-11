# Bosque del Carbonífero — ray tracer CPU

Escena procedural inspirada en el proyecto `esfera`. Conserva sus principios:

- rayos por píxel y cámara perspectiva;
- intersecciones matemáticas con esferas, elipsoides, cápsulas, cilindros y plano;
- materiales procedurales, iluminación difusa/especular, sombras, reflejos y niebla;
- framebuffer propio exportado como PPM, sin modelos ni texturas externas.

La escena representa un pantano del Carbonífero con licopodios arborescentes,
Calamites, helechos, troncos húmedos y una Arthropleura segmentada en primer plano.

## Render

```bash
cargo run --release
sips -s format png carboniferous_swamp.ppm --out carboniferous_swamp.png
```

El programa genera un render de 960 × 640 píxeles.

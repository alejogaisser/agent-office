# Revisión del plan: Agent Office útil

Fecha: 2026-11-18. Revisé `brief.md`, `spec.md` (con `## Revisions`), `ux/flows.md` y `ux/prototype.html` (leído como texto, siguiendo los flujos en el markup y los scripts) contra las decisiones del usuario en la intake y en los Gates 1 a 3, y contra lo que ya existe y garantiza el producto (`README.md`, `docs/plan.md`, `docs/plan-v2.md` y `docs/decisions/`). No hay archivo de contexto de producto.

## Verdict

**Abierto.** Hay 1 hallazgo `blocking` y 15 `major`. El problema, el alcance y la trazabilidad están bien armados, y casi todas las decisiones del usuario están reflejadas. Pero hay huecos que se van a pagar con retrabajo:
- La escritura opt-in en `settings.json` choca cuando las dos opciones conviven.
- El cálculo de la ventana de 5 h está mal definido.
- `AskUserQuestion` es más rico de lo que el plan supone.
- No está definido cuándo termina una espera «Terminó».
- Dos de las tres métricas no van a poder separar el éxito del ruido.

## Findings

### Blocking

**F1 · blocking · spec · `spec.md` > Constraints (punto 3), AC40, AC48; `ux/flows.md` > F10, F14 y Assumptions 9.**
- **Qué está mal:** las dos opciones opt-in son «independientes», pero el deshacer está definido como «restaura el archivo exacto desde el backup» (AC40a: «mismo hash que el backup»), y la UX usa una sola ruta de backup (`settings.json.agent-office.bak`) para las dos. Si el usuario activa el % real, después el control, y deshace el % real, pasa una de dos cosas:
  - Restaurar el backup exacto borra los hooks del control, y la app sigue creyendo que el control está activo.
  - El segundo backup pisa al primero, y deshacer restaura un archivo que ya traía hooks.

  Así como están, AC40a y AC48 no se pueden cumplir a la vez con las dos opciones activas, y el riesgo cae justo sobre la garantía más sensible: no romper la configuración del usuario.
- **Arreglo:**
  - Un backup por opción, con nombres distintos.
  - Una regla explícita: si la otra opción está activa, o si el archivo cambió desde la activación (incluido el cambio que hizo la otra opción), deshacer quita solo las claves propias y no restaura el archivo entero.
  - Un AC nuevo con las dos secuencias (activar A, activar B, deshacer A, deshacer B, y el orden inverso). Pass: después de cada paso, la otra opción sigue funcionando, y al final el hash es igual al original.

### Major

**F2 · major · spec · `spec.md` > Constraints (punto 2, «solo agrega claves, sin tocar las existentes»), AC37, AC39, AC40b; `ux/prototype.html` > diálogo «Activar el % real del límite».**
- **Qué está mal:** encadenar una `statusLine` existente (decisión del Gate 2) obliga a **reemplazar** la clave existente por el comando de la app. Eso contradice el punto (2) y la garantía del diálogo («Solo agrega la clave statusLine; no toca las demás»). Además, el plan no dice:
  - dónde se guarda el comando original para encadenarlo (¿en `config.json` de la app? ¿dentro del nuevo `command`?);
  - qué pasa si el usuario edita su `statusLine` después de activar (hoy el encadenador seguiría llamando al comando viejo, y AC40b «restaura la `statusLine` original» le pisaría el cambio).
- **Arreglo:**
  - Reescribir el punto (2): «agrega claves; si ya existe `statusLine`, la reemplaza por el encadenador y guarda el original en X».
  - Definir el deshacer cuando la propia `statusLine` cambió después de activar (no tocarla y avisar).
  - Corregir la copia del diálogo para el caso con barra propia.

**F3 · major · spec · `spec.md` > A3, AC12, US3.**
- **Qué está mal:** «La ventana actual empieza con la primera actividad posterior al último reset conocido» es incorrecto cuando el último choque fue hace días: después de ese reset hubo muchas ventanas de 5 h. Con esa regla, el inicio de la ventana, los tokens y la hora de «se libera ~HH:MM» salen mal. Tampoco está definido qué tramo suma cada choque para calibrar («las 5 h previas» puede cruzar dos ventanas). Lo correcto es el tramo `[resetsAt − 5 h, hora del rechazo]`.
- **Arreglo:**
  - Definir la ventana como un bloque rodante: arranca con la primera actividad después del fin del bloque anterior y dura 5 h. Un `resetsAt` real, cuando existe, fija el bloque.
  - Definir el tramo de calibración.
  - Agregar a AC12 un caso con el último choque hace más de 5 h y actividad intermedia.

**F4 · major · spec · `spec.md` > A1, AC12, AC29; `docs/plan.md` (líneas 25 y 101, lectura inicial acotada) y `README.md` (`activeWindowMinutes`, `maxInitialReadBytes`).**
- **Qué está mal:** calibrar con los 15 choques exige sumar tokens de semanas atrás, en todos los proyectos y en todos los subagentes. El core actual hace otra cosa:
  - lee como máximo los últimos 8 MiB de cada transcript;
  - no parsea subagentes terminados fuera de la ventana («their tokens are unknown»);
  - solo sigue las sesiones activas en los últimos 30 min.

  El spec no dice que haga falta un escaneo histórico nuevo, ni cuánto cuesta al arrancar (30 días de transcripts). AC29 se mide con escenarios simulados chicos, así que no lo detecta.
- **Arreglo:**
  - Agregar a Constraints o al Build handoff que la calibración necesita un escaneo completo y separado del seguimiento en vivo, con un tope de tiempo y de memoria al arrancar.
  - Agregar un AC de rendimiento sobre un fixture del tamaño real (por ejemplo, 30 días y unos 100 transcripts) con CPU y memoria dentro de AC29 y un tiempo máximo hasta mostrar el %.

**F5 · major · spec · `spec.md` > US13, US15, A8, AC44, AC50; `ux/flows.md` > F12, F15.**
- **Qué está mal:** `AskUserQuestion` puede traer varias preguntas en una sola llamada, preguntas `multiSelect` y la opción «Otra». El plan y el prototipo suponen una sola pregunta de elección única (radio). Ni el spec ni la UX dicen qué hacer con varias preguntas o con selección múltiple, ni cómo arman las `answers` del hook en esos casos.
- **Arreglo:** definir el comportamiento (soportarlos, o mostrarlos en solo lectura con el motivo «contestá en Claude») y agregar un caso a AC44 y a AC50 con 2 preguntas y una `multiSelect`.

**F6 · major · spec · `spec.md` > US1, AC1, AC2; `ux/flows.md` > F1; `README.md` > `activeWindowMinutes`.**
- **Qué está mal:** el ciclo de vida de «Terminó» no está definido. Según el plan, sale de la lista solo «cuando le escribas». Entonces:
  - Toda sesión que terminó y que el usuario no retomó queda «te espera» para siempre. Eso es ruido, porque la app de escritorio mantiene vivas varias sesiones.
  - O, por `activeWindowMinutes = 30`, la sesión desaparece de la app a los 30 min, que es justo la espera larga y no vista que el producto quiere mostrar.

  El prototipo tiene una sesión `idle` que no espera, pero el plan no dice qué la distingue de «Terminó».
- **Arreglo:**
  - Definir cuándo «Terminó» deja de contar (por ejemplo, después de N horas, o cuando el usuario la abre o la descarta) y cómo convive con `activeWindowMinutes`, de modo que una sesión en espera siga visible.
  - Agregar un AC con una espera «Terminó» de 45 min.

**F7 · major · spec · `spec.md` > A4, R7, Paso 0B (B1); AC10.**
- **Qué está mal:** las notificaciones por «Pregunta» vienen apagadas porque «Claude ya avisa» (A4). Pero con el control activo, el hook retiene la pregunta, y R7 admite que el diálogo de escritorio podría no aparecer mientras tanto. Si no aparece el diálogo, probablemente tampoco llegue el aviso de Claude. En ese caso, con el control activo, nadie avisa de las preguntas.
- **Arreglo:**
  - Que el Paso 0B (B1) anote si la notificación de Claude llega mientras el hook espera.
  - Si no llega, el aviso por «Pregunta» de Agent Office se prende solo mientras el control esté activo.
  - Agregar el caso a AC10.

**F8 · major · spec · `spec.md` > R9, AC46, AC47; `ux/flows.md` > F12 (E3), F13.**
- **Qué está mal:** no hay ningún AC sobre qué pasa cuando el hook llega a su timeout con el usuario ausente, ni cuando el canal se cae con Agent Office abierto. La UX promete «tu sesión no queda trabada» y ofrece «Reintentar canal», pero:
  - ningún criterio lo verifica;
  - no hay un estado de UX para «se venció el tiempo, la pregunta pasó a Claude»;
  - el timeout queda en manos del architect sin un rango, y eso decide entre trabar la sesión (si es largo) o volver inútil el control (si es corto).
- **Arreglo:**
  - Fijar un rango para el timeout (o hacerlo configurable, con un valor por defecto).
  - Agregar un AC: con el canal caído y con el timeout vencido, la sesión muestra el diálogo de Claude en N s y la tarjeta pasa a solo lectura con un motivo propio.
  - Sumar ese motivo a los RO_MSG de la UX.

**F9 · major · spec · `spec.md` > US2, AC9a; `ux/prototype.html` > `deliver()` (solo notifica si `S.minimized`).**
- **Qué está mal:** la regla de cuándo notificar solo cubre «enfocada» (AC9a) y «minimizada» (AC7). Quedan sin definir dos de los modos de uso que pidió el usuario: la ventana visible al costado pero sin foco, y la ventana tapada por otras. El prototipo notifica solo si está minimizada, así que con Agent Office tapado detrás del editor no hay aviso.
- **Arreglo:** definir la regla («notificar salvo que la ventana tenga el foco», o la que se elija) y agregar casos a AC9 con la ventana visible sin foco y con la ventana tapada.

**F10 · major · spec · `spec.md` > Success metrics («Uso como tablero diario»), US7, AC23.**
- **Qué está mal:** la métrica cuenta días con al menos un foco o un clic. Abrir la app ya registra un foco (el prototipo lo anota en `init`), así que el 80 % se cumple con solo arrancarla. Y en el modo «ventana lateral» el usuario la mira sin darle foco, así que el uso real no aparece. La métrica no separa «tablero» de «juguete», que es el riesgo 5 del brief.
- **Arreglo:** contar acciones con intención (clic en una tarjeta, un marcador o una notificación, cambio de pestaña, «Ir a Claude», respuesta) y excluir el foco del arranque. Fijar el objetivo sobre esas acciones (por ejemplo, 3 o más por día de actividad, en el 80 % de los días).

**F11 · major · spec · `spec.md` > Success metrics («Esperas largas a preguntas»).**
- **Qué está mal:**
  - Solo mide las esperas por pregunta. Las esperas por fin de turno, que el brief señala como la espera típica, y las de permiso no se miden.
  - Los avisos de «Pregunta» vienen apagados, y la única palanca del MVP sobre las preguntas es US13. Si «responder» falla en el Paso 0B, el objetivo de −50 % no depende de nada que el MVP haga, y la métrica no dice nada.
- **Arreglo:** sumar el P75 de «fin de turno → siguiente mensaje del usuario» (sale de los transcripts, sin tokens) y declarar qué objetivo vale si el Paso 0B falla.

**F12 · major · spec · `spec.md` > A2, A9, AC32, AC34, AC35.**
- **Qué está mal:**
  - AC34 exige que «las partes suman exactamente el total del agente», pero una de las partes, el informe, es una estimación por caracteres (A9: «~1,5 k»), y A9 dice que el desglose de US10 es exacto.
  - No está definido qué es el «total del agente»: ¿la suma de `usage` de todas sus llamadas, contando la caché leída? ¿El tamaño final del contexto?
  - No está definido cómo se calcula «crecimiento del contexto». Si es el resto, la suma exacta queda garantizada por construcción y no prueba nada.
  - AC32 usa «% de los tokens de la sesión» sin decir con qué fórmula.
- **Arreglo:** definir cada categoría con una fórmula sobre `message.usage`, sacar el informe de los `output_tokens` del último mensaje del subagente (exacto) o dejarlo fuera de la suma, y fijar el denominador de AC32 con la fórmula de A2.

**F13 · major · spec · `spec.md` > `## Revisions` y Out of scope 10.**
- **Qué está mal:** el Gate 3 aprobó agregados de la UX que el spec no recoge. No están en Revisions ni tienen AC:
  - píldoras en la escena para las otras sesiones que esperan;
  - la píldora del panel colapsado con las esperas y el %;
  - «Ir a Claude» en las tarjetas «Terminó»;
  - tarjetas de solo lectura en Opciones para los avisos y el registro de uso (Out of scope 10 sigue diciendo que la app «solo tiene los controles» de las opt-in);
  - la regla de que una pregunta que llegó antes de activar queda en solo lectura;
  - el estado «canal caído».

  El architect conserva AC1 a AC51 y nada más, así que todo eso se construye sin criterio.
- **Arreglo:** agregar una entrada del Gate 3 en Revisions, ajustar Out of scope 10 y sumar ACs nuevos (AC52 en adelante) para cada agregado.

**F14 · major · spec · `spec.md` > AC45; `ux/flows.md` > F13 y Assumptions 3.**
- **Qué está mal:** «Aprobar» actúa al primer toque, y el usuario solo ve los primeros 60 caracteres del comando. La cola oculta puede cambiar por completo lo que se aprueba (por ejemplo, `… && rm -rf`). El recorte de 60 caracteres viene de la línea de tiempo y de la privacidad de las notificaciones. En el panel no hace falta, porque lo que se ve en pantalla no se guarda.
- **Arreglo:** en la tarjeta de permiso interactiva, mostrar el comando completo (con scroll) y conservar el recorte solo en la línea de tiempo. Ajustar AC45.

**F15 · major · spec · `spec.md` > AC8, Build handoff (plugin de notificaciones), Assumptions and risks; `README.md` («no Tauri plugins are enabled»).**
- **Qué está mal:** AC8 exige que un clic en el toast restaure la app con la sesión elegida. En Windows, eso pide activación de toasts con un AUMID registrado por el instalador. No está verificado que el plugin de notificaciones de Tauri entregue el clic en escritorio, y en `tauri dev` los toasts pueden salir a nombre de otro proceso. No está entre los riesgos ni en los Pasos 0.
- **Arreglo:** agregar un riesgo y una prueba corta de factibilidad («toast con clic que vuelve a la app con un argumento») antes de la fase 2, con un plan B (el clic solo restaura la app con la pestaña Ahora).

**F16 · major · spec · `spec.md` > Constraints (configuración del usuario), AC46.**
- **Qué está mal:** no está definido qué pasa si Agent Office se desinstala, o si se borra o mueve su helper, con alguna opción activa. Los hooks y la `statusLine` encadenada quedan apuntando a un ejecutable que no existe. El usuario pierde su barra propia y cada pregunta o permiso lanza un comando que falla. AC46 cubre «Agent Office cerrado», no «sin instalar».
- **Arreglo:** que el desinstalador ejecute el deshacer de las dos opciones, o que el encadenado sea robusto a la falta del helper. Agregar un AC: desinstalar con las dos opciones activas deja `settings.json` como el original, o la barra del usuario funcionando.

### Minor

**F17 · minor · spec · `spec.md` > AC31; `README.md` (líneas 3, 9, 10 y 12); `docs/plan.md` (línea 100).**
- **Qué está mal:** AC31 solo pide reescribir «Nothing is persisted». Hay otras frases del README que dejan de ser ciertas:
  - «does nothing while its window is hidden» (y el worker de v1 «only marks itself dirty» mientras está oculta, así que las esperas y las alertas tienen que calcularse en el host mientras está minimizada);
  - «never modifies… ~/.claude» (ahora es «por defecto»);
  - «no Tauri plugins are enabled»;
  - «The only files it writes» (falta el registro de uso).
- **Arreglo:** listar esas frases en AC31 y dejar escrito en el Build handoff que la detección y los avisos corren en el host con la ventana oculta.

**F18 · minor · spec · `spec.md` > AC28.**
- **Qué está mal:** el filtro de Process Monitor es «Process Name is agent-office.exe», pero el mismo criterio dice que incluye los procesos de la status line y del hook, que tienen otro nombre.
- **Arreglo:** agregar los nombres de los helpers al filtro.

**F19 · minor · spec · `spec.md` > Build handoff (fases 2 y 3).**
- **Qué está mal:** la fase 2 dice que AC23 y AC25 «se cierran en la fase 3», pero la condición de fin de la fase 3 no los lista.
- **Arreglo:** sumarlos a la fase 3.

**F20 · minor · user · `spec.md` > A7, AC20.**
- **Qué está mal:** al abrir la app, todo cuenta como «nuevo». En el modo «abrirla de vez en cuando», la marca no ayuda a ver «qué pasó mientras no estaba». El registro de uso (US7) ya guarda la hora del último foco sin contenido y podría servir de referencia.
- **Arreglo:** que decida el usuario (ver la pregunta del reporte) y ajustar A7 y AC20.

**F21 · minor · spec · `spec.md` > AC24, Success metrics.**
- **Qué está mal:** el registro rota a 1 MB, sin decir si conserva al menos 30 días, que es lo que necesita la métrica de choques. Además, la métrica de choques se lee a los 30 días, justo cuando `cleanupPeriodDays` (30) empieza a borrar los transcripts del día 0.
- **Arreglo:** que la rotación conserve al menos 30 días (o un archivo anterior) y leer la métrica el día 28, o semana a semana.

**F22 · minor · ux · `ux/prototype.html` > `releaseToClaude()`; `ux/flows.md` > F12 E1, F3.**
- **Qué está mal:**
  - Después de «Contestar en Claude», la tarjeta muestra el motivo «llegó antes de activar el control», que es falso. F12 E1 no define el estado de la tarjeta después de liberar.
  - No está definido adónde lleva el clic en el toast de la alerta de límite (el prototipo restaura sin sesión).
- **Arreglo:** un motivo propio («la mandaste a Claude») y, para la alerta de límite, abrir Ahora > Límite de 5 h.

**F23 · minor · spec · `spec.md` > AC17; `ux/flows.md` > F4.**
- **Qué está mal:**
  - AC17 espera «+4 agentes» y F4 dice «+10 agentes» (son los datos del prototipo).
  - AC17 no dice si Main cuenta como agente (el prototipo lo cuenta).
  - Tampoco dice cómo se nombran dos agentes con el mismo rol en una sesión.
- **Arreglo:** definirlo en AC17 y alinear el texto de F4.

**F24 · minor · spec · `spec.md` > AC37, AC43, AC42.**
- **Qué está mal:**
  - «La única diferencia» no dice si es textual o semántica: reserializar el JSON cambia los espacios y el orden de las claves.
  - Falta el caso en que `settings.json` no existe.
- **Arreglo:** decir si se preserva el formato o si alcanza la igualdad semántica, y definir el caso sin archivo (crearlo, y que deshacer lo borre).

**F25 · minor · spec · `spec.md` > Paso 0A.**
- **Qué está mal:** la prueba empieza con el `settings.json` del proyecto, pero el producto escribe en el del usuario. Que pase en el del proyecto no prueba el caso real.
- **Arreglo:** que «pasa» exija probar también en `~/.claude/settings.json` (con backup), o que la app escriba en el mismo nivel que se probó.

**F26 · minor · spec · `spec.md` > AC30; `ux/flows.md` > Assumptions 13.**
- **Qué está mal:** el modo «ventana lateral» de la intake no tiene ningún criterio con el panel abierto en una ventana angosta. AC30 mide 1200x760 abierto y 900x600 colapsado, y el prototipo pasa a hoja inferior a 900 px o menos.
- **Arreglo:** agregar un tamaño de ventana lateral (por ejemplo, 600x900) a AC30 con el panel abierto y «Te esperan» visible sin scroll.

## Traceability

| Historia | Criterios | Pantallas | Flujos | Huecos |
|---|---|---|---|---|
| US1 (must) | AC1–AC6, AC26–AC30 | S2, S3, S4, S5, S16 | F1 | F6 (ciclo de «Terminó»), F13 (píldoras sin AC) |
| US2 (must) | AC7–AC11 | S4, S17, S11 | F2 | F9 (visible sin foco o tapada), F15 (clic en el toast) |
| US3 (must) | AC12–AC14 | S6, S16 | F3 | F3 (algoritmo de la ventana), F4 (escaneo histórico) |
| US4 (must) | AC15, AC16 | S6, S17 | F3 | F22 (destino del clic en la alerta) |
| US5 (must) | AC17, AC18 | S7 | F4 | F23 |
| US6 (must) | AC19–AC22 | S8 | F5 | F20 (qué cuenta como «nuevo») |
| US7 (must) | AC23–AC25 | S11 | F2, F8 | F10, F21 |
| US8 (must, Paso 0A) | AC37–AC42, AC51 | S6, S11, S12, S14, S15 | F9, F10 | F1, F2, F16, F24, F25 |
| US9 (must) | AC32, AC33 | S9 | F6 | F12 (denominador) |
| US10 (must) | AC34–AC36 | S7, S10 | F4, F7 | F12 (fórmulas) |
| US13 (must, Paso 0B) | AC43–AC49, AC51 | S5, S11, S13, S14, S15 | F11–F14 | F1, F5, F7, F8, F14, F16 |
| US15 (must) | AC50 | S5 | F15 | F5 (varias preguntas), F13 («Ir a Claude» en «Terminó» sin AC) |

Todas las historias del MVP tienen al menos un criterio y al menos una pantalla y un flujo. Ninguna cadena está cortada; los huecos son de definición dentro de cada eslabón.

## Strengths

- La trazabilidad del spec a la UX es completa, y cada flujo tiene sus caminos de error con copia que dice qué hacer.
- Los riesgos más caros (el % real y el control) están puestos detrás de pruebas baratas (Pasos 0A y 0B), con un respaldo de solo lectura (US15) que deja el MVP útil aunque fallen.
- La evidencia del brief reorientó bien el pedido de «comunicación concisa» hacia «dónde se van los tokens» (Out of scope 4 y US10).

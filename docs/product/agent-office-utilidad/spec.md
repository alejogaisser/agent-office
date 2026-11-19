<!-- product-team spec -->
# Agent Office útil: esperas, límite de 5 h y qué pasó

## Summary

- **Problema:** un dev solo corre varias sesiones de Claude Code en paralelo y tiene tres problemas. No se entera a tiempo de qué sesión lo espera. Choca con el límite de 5 h sin aviso (hay 15 choques registrados, 6 en los últimos 30 días). Y al volver tiene que releer cada chat.
- **Usuario primario:** el propio usuario, en Windows 11, con la app de escritorio de Claude y los equipos de `gaisser-agents`.
- **Resultado:** Agent Office pasa a ser su tablero. Le avisa quién lo espera aunque esté minimizado, le muestra cuánto lleva del límite y quién lo consume, le resume qué pasó en cada sesión y le muestra cómo fluye el trabajo entre agentes y dónde se van los tokens. Si activa las opciones opt-in, también ve el % real del límite y contesta o aprueba desde la app. No gasta tokens, no usa red y por defecto no escribe nada en `~/.claude`.

## User stories

- US1 (`must`): Como dev con varias sesiones abiertas, quiero ver en un panel al costado del 3D, y con un marcador dentro del 3D, qué sesiones me esperan, por qué y desde hace cuánto, para contestar primero a la que más espera.
- US2 (`must`): Como dev que tiene Agent Office minimizado, quiero una notificación de Windows cuando una sesión pasa a esperarme por un motivo que no me avisa ya la app de escritorio (o por uno que yo prenda), para no tener que estar mirando la ventana.
- US3 (`must`): Como dev con plan por suscripción, quiero ver cuántos tokens llevo en la ventana actual de 5 h, un % estimado del límite y una hora estimada de reset, para no chocar por sorpresa.
- US4 (`must`): Como dev, quiero una alerta cuando el % estimado pasa un umbral (80 % por defecto), para pausar, compactar o cerrar sesiones antes de chocar.
- US5 (`must`): Como dev, quiero ver qué sesiones y qué agentes consumieron más tokens en la ventana actual, para saber qué me consume el límite.
- US6 (`must`): Como dev que vuelve después de un rato, quiero una línea de tiempo por sesión con lo que pasó (subagentes lanzados, informes, archivos editados, comandos, errores, choques con el límite, compactaciones), con lo nuevo marcado, para retomar sin releer el chat.
- US7 (`must`): Como usuario de Agent Office, quiero que la app anote en su propio log local, sin contenido de mis sesiones, cuándo me avisó y cuándo la miré, para saber en dos semanas si me sirve.
- US8 (`must`, solo si pasa el Paso 0A): Como dev, quiero el % real del límite de 5 h (y el de 7 días) con la hora exacta de reset, activando una opción que escribe la status line en `~/.claude/settings.json` con backup y botón de deshacer, y que encadena mi status line existente, para dejar de depender de una estimación.
- US9 (`must`): Como dev, quiero ver el flujo entre agentes de una sesión (quién lanzó a quién, quién le informó a quién y el tamaño de cada mensaje), para entender si los agentes se hablan entre sí y cuánto pesa esa comunicación.
- US10 (`must`): Como dev, quiero ver dónde se van los tokens por agente y por sesión (arranque, crecimiento del contexto, reescrituras de caché después de pausas, informes devueltos a Main, cantidad de subagentes lanzados y modelo), para saber qué cambiar para ahorrar.
- US11 (`could`): Como dev, quiero una proyección del tipo «a este ritmo chocás a las HH:MM», para anticiparme antes de llegar al umbral.
- US12 (`could`): Como dev, quiero consejos de ahorro basados en reglas fijas, sin LLM (por ejemplo, «este subagente está parado hace más de 5 min: al retomar reescribe unos N tokens de caché»), para actuar sobre las palancas grandes.
- US13 (`must`, por partes según el Paso 0B): Como dev, quiero aprobar permisos y responder preguntas de Main (eligiendo una de sus opciones) desde Agent Office, activando hooks y un canal local opcionales, para no tener que cambiar de ventana.
- US14 (`could`): Como dev, quiero indicadores de consumo dentro del 3D (por ejemplo, un medidor por edificio o por sala), para ver el costo sin abrir el panel.
- US15 (`must`; se usa siempre que US13 no esté disponible: con la opción de control apagada o para la parte que falle en el Paso 0B): Como dev, quiero ver en Agent Office la pregunta o el permiso pendiente en solo lectura, con sus opciones, y un botón «Ir a Claude» que traiga al frente la app de escritorio, para decidir rápido aunque tenga que contestar allá.
- US16 (`could`): Como dev, quiero contestar las preguntas de Main con texto libre (la opción «Otra») desde Agent Office, para no tener que ir a Claude cuando ninguna opción me sirve.

## MVP scope

MVP:
- **Siempre (solo lectura):** US1 (panel y marcador «te espera»), US2 (notificación de Windows), US3 (tokens de la ventana y % estimado), US4 (alerta de umbral), US5 (quién consume), US6 (línea de tiempo por sesión), US7 (log local de uso para medir), US9 (flujo entre agentes) y US10 (dónde se van los tokens).
- **Opt-in, según el Paso 0:** US8 (% real con status line encadenada) si pasa el Paso 0A. US13 (responder y aprobar desde la app) en las partes que pasen el Paso 0B.
- **Respaldo de US13:** US15 (pregunta o permiso en solo lectura, con «Ir a Claude»). Es de solo lectura, así que funciona también con la opción de control apagada y para las partes que fallen el Paso 0B.

**Listo significa:** al abrir el panel ves cuál sesión te espera, qué pasó en ella, cómo se pasaron el trabajo sus agentes, cuánto llevás del límite de 5 h y dónde se fueron los tokens. La app te avisó antes de pasar el 80 %. Con las opciones opt-in apagadas, todo funciona sin tokens, sin red y sin escribir en `~/.claude`. Si las prendés (con aviso, backup y deshacer), ves el % real y contestás o aprobás desde Agent Office (o, si el Paso 0B falló, ves la pregunta y saltás a Claude con un clic).

El orden de construcción está en Build handoff: primero los Pasos 0A, 0B y 0C, después la base de solo lectura y al final las opciones opt-in.

## Later

- US11 (`could`): el umbral de US4 alcanza para evitar la sorpresa. La proyección mejora el aviso, pero agrega otra heurística sin calibrar.
- US12 (`could`): depende de que US10 esté en uso. Primero hay que ver qué causas pesan de verdad.
- US14 (`could`): el MVP ya pone en el 3D el marcador de espera. Los medidores de consumo en la escena pueden esperar a que se use el panel.
- US16 (`could`): elegir una opción cubre la mayoría de las preguntas de Main. El texto libre se agrega si el Paso 0B muestra que el hook acepta cualquier respuesta.

## Out of scope

1. Ícono en la bandeja del sistema y arranque con Windows. Las notificaciones funcionan con la ventana minimizada, y cerrar la ventana cierra la app.
2. Saltar a la sesión exacta dentro de la app de escritorio de Claude: no hay deep link documentado. El clic lleva a Agent Office, no a Claude.
3. Historia propia de largo plazo. La app no guarda sesiones, prompts ni tokens más allá de lo que hay en los transcripts (que se borran a los 30 días). El log de US7 guarda solo marcas de eventos, sin contenido.
4. Acortar o reescribir automáticamente los prompts o informes entre agentes («comunicación concisa»). La evidencia muestra que pesan de 0,5 k a 3 k tokens, frente a los 100 k a 300 k de contexto de cada subagente. En su lugar, US10 muestra dónde se van los tokens.
5. Mandar mensajes a las sesiones por el named pipe de mensajería: cuenta como uso y exige el token de la sesión.
6. Cualquier resumen hecho con un LLM (como el recap nativo) y los hooks `prompt` o `agent`, porque gastan tokens.
7. Cifras en USD como medida principal: para suscriptores no cuentan en la facturación. Lo que cuenta es el % del plan.
8. Red de cualquier tipo, Remote Control, acceso desde el teléfono o la web, y un receptor de OpenTelemetry.
9. macOS, Linux y el segmento secundario (otros usuarios del plugin).
10. Una pantalla completa de ajustes. En el MVP, la pestaña Opciones tiene dos cosas: los controles para prender y deshacer las dos opciones opt-in (status line y control), con su aviso, y tarjetas de solo lectura que muestran el estado de los avisos de Windows y del registro de uso (AC61). Ninguna de esas tarjetas cambia un ajuste: el umbral, los motivos de aviso y el registro se cambian en el archivo de configuración existente.
11. Sonido en las notificaciones más allá del sonido por defecto de Windows.
12. Escribirle un prompt nuevo a una sesión (por ejemplo, contestar un fin de turno) desde Agent Office. Solo se responden preguntas y permisos pendientes, a través de los hooks de US13.
13. Aprobar permisos o responder preguntas en lote, o con reglas automáticas («aprobar siempre X»). Cada decisión la toma el usuario, de a una.

## Success metrics

| Métrica | Cómo se mide | Línea base | Objetivo | Cuándo se lee |
|---|---|---|---|---|
| Esperas largas | Desde los transcripts, sin tokens, dos lecturas: (1) tiempo entre cada `AskUserQuestion` de Main y la respuesta del usuario; (2) tiempo entre cada fin de turno de Main que cuenta como «Terminó» (AC5) y el siguiente mensaje del usuario en esa sesión. Para cada una se reportan el P75 y el % de esperas de más de 5 min (comando de AC25). | Se calcula con los 14 días previos a usar el MVP, con los transcripts que ya hay en disco. Hoy `unknown`; en una muestra de 5 preguntas con el usuario presente, la espera fue de 31 s a 4,2 min. | Preguntas: P75 un 50 % menor si «responder» pasó el Paso 0B, o un 25 % menor si no pasó (solo US15). Fin de turno: P75 un 25 % menor. | 14 días después de empezar a usar el MVP a diario |
| Choques con el límite sin aviso | Rechazos `five_hour` en los transcripts que no tuvieron una alerta de US4 en el log de US7 al menos 15 min antes | 6 choques en 30 días, todos sin aviso | 1 o menos en 30 días | 30 días después de empezar a usar el MVP |
| Uso como tablero diario | Días con 3 o más acciones con intención (log de US7), dividido por los días con actividad de Claude Code (transcripts). Cuentan como acción con intención: un clic en una tarjeta, una píldora, el marcador o una notificación; un cambio de pestaña; «Ir a Claude»; una respuesta, aprobación o rechazo. No cuentan el foco al arrancar ni el foco solo. | `unknown` | 80 % o más | 14 días después de empezar a usar el MVP |

## Acceptance criteria

Los comandos se corren desde `C:\dev\agent-office`. «Escenario simulado» se refiere a `node scripts/simulate.mjs --root <tmp> --scenario <nombre>`, con la app apuntada a `<tmp>` por `AGENT_OFFICE_CLAUDE_DIR`. El architect agrega los escenarios y fixtures nuevos con los nombres que elija; los nombres de abajo son sugerencias. Las referencias v1-ACn y v2-ACn apuntan a `docs/plan.md` y `docs/plan-v2.md`.

**US1: te esperan**
- AC1 (US1): La sección «Te esperan» lista solo las sesiones que esperan al usuario, ordenadas de la que más espera a la que menos, con el motivo y el tiempo. Check: escenario `waiting` con 3 sesiones; en t0 la sesión B hace una pregunta (`AskUserQuestion` sin respuesta), en t0+60 s la sesión C termina su turno y la sesión A sigue trabajando. Mirar el panel en t0+90 s. Pass: primero B con «Pregunta» y unos 1:30, después C con «Terminó» y unos 0:30 (±5 s); A no aparece.
- AC2 (US1): Una sesión sale de la lista cuando el usuario le contesta. Check: en el escenario de AC1, el simulador escribe la respuesta del usuario en B. Pass: B desaparece de «Te esperan» en 2 s o menos; C sigue.
- AC3 (US1): El 3D y la barra lateral muestran la espera. Check: en el escenario de AC1, seleccionar el edificio B y después el C. Pass: sobre la oficina del jefe del piso seleccionado hay un marcador visible, distinto para «Pregunta» y para «Terminó», con texto y no solo color. En la barra lateral, B y C muestran «te espera» y el tiempo. A no tiene marcador.
- AC4 (US1): Un clic en la lista lleva a la sesión. Check: hacer clic en C dentro de «Te esperan». Pass: en 500 ms o menos, C queda seleccionado en la barra lateral y en el 3D.
- AC5 (US1, borde): Un fin de turno de Main mientras sus subagentes trabajan no cuenta como espera. Check: escenario `waiting-background`: Main termina su turno con un subagente en segundo plano trabajando; 60 s después el subagente termina y Main cierra su turno sin lanzar otro. Pass: durante los primeros 60 s la sesión no aparece en «Te esperan» y no hay notificación; aparece con «Terminó» solo después del cierre final de Main.
- AC6 (US1, borde): Una sesión que se cierra mientras espera sale de la lista. Check: en el escenario de AC1, terminar el proceso simulado de C. Pass: C sale de «Te esperan» a más tardar `closedGraceSeconds` + 2 s después. Sin sesiones en espera, la sección muestra «Nadie te espera».

**US2: notificación de Windows**
- AC7 (US2): Una espera por permiso genera una notificación de Windows aunque la app esté minimizada. Check: minimizar Agent Office, correr el escenario `waiting-permission` (la sesión B queda con una herramienta pendiente de permiso, que no es `AskUserQuestion`, y el registro en `waiting`) y comparar la hora de escritura que imprime el simulador con la hora del aviso en el log de US7. Pass: aparece una notificación por B con el título de la sesión y el motivo («Permiso»), en 5 s o menos, sin el texto de la herramienta, rutas ni comandos. El panel lista B con el motivo «Permiso».
- AC8 (US2): Un clic en la notificación lleva a la sesión. Check: con la app instalada (no en `tauri dev`), hacer clic en la notificación de AC7. Pass: si pasó el Paso 0C, Agent Office se restaura al frente con B seleccionada y la pestaña Ahora abierta, en 1 s o menos. Si el Paso 0C falló, se restaura al frente con la pestaña Ahora abierta y B primera en «Te esperan», en 1 s o menos.
- AC9 (US2, borde): Se notifica salvo que la ventana tenga el foco, y sin repetir. Check: (a) con la ventana de Agent Office enfocada, correr el escenario de AC7; (b) minimizada, escenario `waiting-burst`, donde 3 sesiones entran en espera por permiso dentro de 5 s y se quedan esperando 5 min; (c) con la ventana visible al costado pero con el foco en otra app, correr el escenario de AC7; (d) con la ventana tapada por otra ventana maximizada, correr el escenario de AC7. Pass: (a) 0 notificaciones, y el panel se actualiza igual; (b) exactamente 1 notificación («3 sesiones te esperan») y ninguna más mientras sigan esperando; (c) y (d) 1 notificación cada una.
- AC10 (US2, borde): Los motivos que ya avisa la app de escritorio vienen apagados y se pueden prender. Check: (a) con la configuración por defecto, minimizar la app y correr el escenario de AC1; (b) prender en la configuración las notificaciones por «Pregunta», reiniciar y repetir; (c) solo si el Paso 0B (B1) mostró que Claude no avisa mientras el hook retiene la pregunta: con la configuración por defecto y el control activo, minimizar y provocar una pregunta. Pass: (a) 0 notificaciones, y B (Pregunta) y C (Terminó) aparecen igual en «Te esperan»; (b) 1 notificación por B y ninguna por C; (c) 1 notificación por la pregunta. Al deshacer el control, «Pregunta» vuelve a quedar apagada.
- AC11 (US2, borde): Si Windows bloquea las notificaciones, la app sigue funcionando. Check: desactivar las notificaciones de Agent Office en Configuración > Sistema > Notificaciones y correr el escenario de AC7 con la app minimizada. Pass: la app no se cierra ni muestra error, el panel lista B y el log tiene una sola advertencia de notificación no entregada.

**US3: ventana de 5 h y % estimado**
- AC12 (US3): Los tokens de la ventana actual y el % estimado salen de los transcripts de todos los proyectos, incluidas las sesiones ya cerradas, con la ventana y la calibración definidas en A1 y A3. Check: fixture `usage-basic` con 3 choques `five_hour` (con `resetsAt` conocido) cuyos tramos de calibración `[resetsAt − 5 h, hora del rechazo]` suman 1,0 M, 1,2 M y 1,4 M tokens contados, y 600 k tokens en la ventana actual, repartidos en 2 proyectos (uno con la sesión ya cerrada). El tramo del segundo choque arranca con tokens que pertenecen a una ventana anterior. Además, fixture `usage-old-hit`: el último choque fue hace 2 días y después hubo 3 bloques de actividad separados por más de 5 h. Correr `cargo test -p office-core usage_estimate` y abrir la app sobre cada fixture. Pass:
  - El test pasa.
  - En `usage-basic`, el panel muestra «600 k tokens», «~50 % (estimado)» (techo = mediana de los choques, 1,2 M) y «se libera ~HH:MM (estimado)», con la hora igual al inicio del bloque actual + 5 h. Los tokens de la ventana anterior no entran en la calibración del segundo choque.
  - En `usage-old-hit`, la ventana actual empieza con la primera actividad del tercer bloque, no con el reset del último choque, y los tokens y la hora de reset salen de ese bloque.
- AC13 (US3): Si hay un choque en la ventana actual, se usa la hora real de reset. Check: el simulador agrega un rechazo `five_hour` con `resetsAt` = 16:00 en la ventana actual. Pass: en 2 s o menos, el panel muestra «Límite alcanzado · se libera 16:00», sin la palabra «estimado».
- AC14 (US3, borde): Sin choques registrados no se inventa un %. Check: fixture `usage-no-hits`, con tokens en la ventana y ningún rechazo en disco. Pass: el panel muestra los tokens de la ventana y «Sin choques registrados para estimar el %». No muestra ningún %, y US4 no dispara alertas.

**US4: alerta de umbral**
- AC15 (US4): Cruzar el umbral genera una sola alerta por ventana. Check: fixture `usage-climb`, que con la calibración de AC12 pasa del 70 % al 85 % estimado y se queda en 90 %, con la app minimizada. Pass: hay exactamente 1 notificación («Llevás ~80 % del límite de 5 h (estimado) · se libera ~HH:MM») y la sección del límite queda resaltada con texto. No hay otra notificación mientras siga arriba del umbral. En la ventana siguiente puede volver a avisar.
- AC16 (US4, borde): El umbral se puede configurar. Check: poner el umbral en 60 en la configuración, reiniciar y repetir AC15. Pass: la única alerta llega al cruzar el 60 %. Un valor inválido (por ejemplo, 150 o texto) usa 80 y deja una advertencia en el log.

**US5: quién consume**
- AC17 (US5): El panel ordena sesiones y agentes por consumo en la ventana actual. Check: fixture `usage-ranking` con 7 sesiones y 9 agentes con tokens conocidos. Pass: se ven las 5 sesiones y los 5 agentes que más consumieron (agente = rol + título de la sesión), en orden descendente, con los tokens y el % del total de la ventana, exactos según el fixture. Abajo dice «+2 sesiones» y «+4 agentes».
- AC18 (US5, borde): Solo se cuenta lo que cae dentro de la ventana. Check: en el mismo fixture, un agente con consumo solo antes del inicio de la ventana y otro con consumo antes y dentro. Pass: el primero no aparece; el segundo aparece solo con los tokens de dentro de la ventana.

**US6: qué pasó**
- AC19 (US6): La línea de tiempo muestra los eventos de la sesión. Check: escenario `timeline` con la secuencia: lanzamiento de software-architect, Edit de un archivo, comando Bash, informe del architect, error de API, rechazo `five_hour` y compactación. Seleccionar la sesión. Pass: hay 7 entradas, de la más nueva a la más vieja, cada una con hora HH:MM, agente y resumen. El informe muestra su primera línea (120 caracteres como máximo), el archivo su ruta relativa al proyecto y el comando sus primeros 60 caracteres.
- AC20 (US6): Lo nuevo se cuenta desde la última vez que miraste la app (A7). Check: (a) con la app enfocada y la sesión A en pantalla, minimizarla; el simulador escribe 2 eventos en A; restaurar la app y abrir A. (b) Cerrar la app; el simulador escribe 1 evento en A; volver a abrir la app y abrir A. (c) Apagar el log de uso, reiniciar y abrir A. Pass:
  - (a) Exactamente esos 2 eventos llevan la marca «nuevo».
  - (b) Solo el evento escrito con la app cerrada lleva la marca, porque la hora de referencia sale del log de uso y sobrevive al reinicio.
  - (c) Todos los eventos que hay al abrir llevan la marca, sin error.
- AC21 (US6, borde): Un transcript más grande que la lectura inicial muestra que la historia es parcial. Check: escenario con un transcript de más de `maxInitialReadBytes`. Pass: la línea de tiempo empieza con «Historia parcial: desde HH:MM», sin error. Una línea con un tipo desconocido o JSON roto se ignora y los eventos siguientes aparecen.
- AC22 (US6, borde): La línea de tiempo no se guarda. Check: cerrar y volver a abrir la app sobre el mismo escenario y listar los archivos que escribió la app en `%APPDATA%\agent-office\` y `%LOCALAPPDATA%\agent-office\`. Pass: la línea de tiempo se reconstruye igual desde los transcripts. Solo existen la configuración y los logs, y ninguno contiene textos de informes, rutas ni comandos del escenario.

**US7: log local de uso**
- AC23 (US7): El log registra solo marcas de eventos. Check: correr AC7, AC8, AC15, AC56 y AC50, cambiar de pestaña una vez y después buscar en el log de uso con `Select-String` las rutas, los comandos y los textos de informe y de pregunta del escenario. Pass:
  - Hay una línea por cada uno de estos eventos: notificación emitida, alerta de umbral, foco de la ventana (marcando si fue al arrancar), pérdida de foco o minimizado, y cada acción con intención de la métrica de uso (clic en una tarjeta, una píldora, el marcador o una notificación; cambio de pestaña; «Ir a Claude»; respuesta, aprobación o rechazo).
  - Cada línea tiene hora, tipo y, si corresponde, el id de la sesión.
  - La búsqueda da 0 coincidencias.
- AC24 (US7, borde): El log tiene un tamaño acotado y se puede apagar. Check: (a) escenario `soak` durante 10 min; (b) apagar el log de uso en la configuración y repetir AC7. Pass: (a) el log no pasa de 1 MB (rota); (b) no se escribe ninguna línea de uso y la notificación sale igual.
- AC25 (US7): Las 3 métricas se calculan sin tokens. Check: un comando de `office-dump` (el architect define el flag) con un rango de fechas, corrido sobre un fixture con resultados conocidos. Pass: imprime en JSON, iguales a los esperados del fixture:
  - el P75 y el % de esperas de más de 5 min, por separado para preguntas y para fin de turno;
  - los choques sin aviso;
  - el % de días de actividad con 3 o más acciones con intención, sin contar el foco al arrancar.

**Transversales (todas las historias del MVP)**
- AC26 (US1–US7, US9, US10, US15): Sin red. Check: `pwsh scripts/measure-resources.ps1 -Name agent-office -Seconds 120 -Network` mientras corren AC7 y AC15. Pass: `remoteConnections` del proceso de la app es 0 (el método y la limitación conocida de WebView2 son los de v2-AC11). Los chequeos de v1-AC19 siguen pasando.
- AC27 (US1–US7, US9, US10, US15): Solo lectura sobre `~/.claude` con las opciones opt-in apagadas, que es el valor por defecto. Check: con el escenario `static` (sin escrituras), guardar `Get-FileHash` de todos los archivos de `<tmp>` y del `settings.json` real, usar la app 10 min (panel, notificaciones, línea de tiempo) y volver a calcular. Pass: los hashes son idénticos y no hay archivos nuevos.
- AC28 (US1–US15): Nunca abre `*.key`. Check: Process Monitor con el filtro «Process Name is agent-office.exe» y «Path ends with .key» durante AC1 a AC7 y AC37 a AC50 (incluidos los procesos de la status line y del hook). Pass: 0 eventos.
- AC29 (US1–US15): Presupuestos. Check: v1-AC15 con el panel abierto y con la app minimizada mientras detecta esperas, y v2-AC22 con el panel abierto, con las opciones opt-in apagadas y otra vez con las dos activas. Pass: CPU de 1,0 % o menos con agentes trabajando y de 0,3 % o menos en reposo o minimizada; memoria privada total de 500 MB o menos y del host de 40 MB o menos.
- AC30 (US1–US7): Regresión de v2. Check: `cargo test -p office-core`, `npm --prefix ui run build` y los v2-AC de render bajo demanda (v2-AC20) y de cámara (v2-AC19), con el panel abierto a 1200x760 y colapsado a 900x600. Pass: todo pasa; el overlay F3 muestra `fit ok` en los dos tamaños, y con el panel abierto y sin cambios no se dibujan frames.
- AC31 (US1–US7): El README dice la verdad. Check: leer `## Data access and privacy`. Pass: menciona las notificaciones de Windows y qué muestran (título y motivo, sin contenido), que Windows puede guardarlas en su centro de notificaciones, el log de uso (qué guarda, dónde y cómo apagarlo) y que el % del límite es una estimación. Además, la frase «Nothing is persisted» se reescribe para que sea exacta.

**US9: flujo entre agentes**
- AC32 (US9): La sección «Flujo» muestra quién lanzó a quién y quién le informó a quién, con el tamaño de cada mensaje. Check: fixture `flow-basic`. Main lanza a1 (architect), a2 (coder) y a3 (tester) con prompts de 4.000, 8.000 y 2.000 caracteres; Main le manda 2 `SendMessage` a a2; a2 le manda 1 `SendMessage` a a3; cada subagente devuelve un informe de 6.000 caracteres. Seleccionar la sesión. Pass:
  - Main aparece en el centro con 3 flechas de lanzamiento: ~1 k, ~2 k y ~0,5 k tokens (estimados a 4 caracteres por token y marcados con «~»).
  - Hay 2 mensajes de Main a a2 y 3 informes de ~1,5 k.
  - La flecha de a2 a a3 está resaltada como «subagente a subagente».
  - Un total dice «Mensajes entre agentes: ~X tokens, Y % de los tokens de la sesión», igual al esperado del fixture. X es la suma de los tamaños estimados de todos los mensajes (A9) e Y = X dividido por los tokens contados de la sesión con la fórmula de A2.
- AC33 (US9, borde): Casos sin subagentes, con anidamiento o con informe pendiente. Check: (a) una sesión solo con Main; (b) un fixture donde un subagente lanza a otro (profundidad 2); (c) un subagente que sigue trabajando. Pass: (a) muestra «Main no lanzó subagentes»; (b) el nieto cuelga de su padre, no de Main; (c) su informe aparece como «pendiente».

**US10: dónde se van los tokens**
- AC34 (US10): El desglose por agente separa las causas con las fórmulas de A11. Check: fixture `tokens-causes` con un subagente de `usage` conocido llamada por llamada: una primera llamada con 12.000 tokens de caché escrita; el contexto crece hasta 314.000; una pausa de 3 h seguida de una llamada con `cache_read_input_tokens` 0 y `cache_creation_input_tokens` 313.639; y un último mensaje con 1.480 `output_tokens`. Pass:
  - Cada categoría es igual al valor calculado a mano en el fixture: arranque 12.000; reescritura de caché tras pausa 313.639 (con la hora y la duración de la pausa); crecimiento del contexto; caché leída; salida; e informe devuelto 1.480 (dentro de la salida, sin sumarse dos veces).
  - Se muestran el modelo y el total del agente, que es la suma de los tokens contados (A2) de todas sus llamadas.
- AC35 (US10): El resumen por sesión y por ventana suma bien. Check: fixture con una sesión de 4 subagentes repartidos en 2 modelos. Pass: la sesión muestra «4 subagentes lanzados», los tokens por modelo y el arranque total (la suma de los 4), iguales al fixture. La sección del límite muestra el mismo resumen para la ventana actual de 5 h, sumando todas las sesiones.
- AC36 (US10, borde): No hay reescrituras falsas ni errores sin datos. Check: (a) una pausa de 4 min en un subagente y una de 50 min en Main, cada una seguida de una llamada que lee de caché; (b) un agente sin líneas `usage`. Pass: (a) no se cuenta ninguna reescritura por pausa; (b) el agente muestra «sin datos de tokens», sin error.

**US8: % real con status line (solo si pasa el Paso 0A)**
- AC37 (US8): Activar avisa antes de escribir. Check: en una copia de prueba de la carpeta de Claude (`AGENT_OFFICE_CLAUDE_DIR`), con un `settings.json` sin `statusLine`, tocar «Activar % real». Pass:
  - Antes de escribir, la app muestra qué va a escribir, en qué archivo, dónde queda el backup y cómo deshacerlo, y pide confirmar.
  - Si se cancela, el hash de `settings.json` no cambia.
  - Si se confirma, queda el backup propio de esta opción (con un nombre distinto del backup del control) con el mismo hash que el original, y la única diferencia en `settings.json` es la clave `statusLine`.
- AC38 (US8): El % real reemplaza la estimación. Check: con la opción activa, pasarle al comando de status line instalado un JSON de ejemplo con `rate_limits.five_hour.used_percentage` 47, `resets_at` 16:00 y `seven_day` 22. Pass: en 2 s o menos, el panel muestra «47 % (real) · se libera 16:00» y «7 días: 22 %», y la alerta de US4 usa el % real. Durante la prueba, el comando no hace conexiones fuera de loopback.
- AC39 (US8): La status line existente sigue funcionando. Check: un `settings.json` con una `statusLine` propia cuyo comando imprime «MI BARRA». Activar la opción y pasarle al comando instalado el JSON de AC38. Pass:
  - El aviso previo dice que la `statusLine` actual se reemplaza por el encadenador de Agent Office, que la sigue llamando, y dónde se guarda el comando original.
  - En `settings.json`, la única diferencia es el valor de `statusLine`. El comando original queda guardado en la configuración propia de la app y en el backup de esta opción.
  - La salida es exactamente «MI BARRA» y la app recibe el 47 %.
  - Si el comando original falla o tarda más que su límite, la salida es la misma que daría el original solo (vacía o con su error), y la app recibe el dato igual.
- AC40 (US8): Deshacer restaura sin pisar cambios del usuario ni la otra opción. Check: con el control apagado, tocar «Deshacer % real» en tres casos: (a) sin cambios en el archivo desde la activación; (b) después de agregar otra clave a mano; (c) después de que el usuario reemplazó a mano la `statusLine` del encadenador por otra. Pass:
  - (a) `settings.json` queda con el mismo hash que el backup de esta opción.
  - (b) Se quita solo lo que agregó la app, se restaura la `statusLine` original y se conserva la clave nueva. La app avisa que no restauró el archivo entero.
  - (c) La app no toca la `statusLine` del usuario, quita solo lo demás que haya agregado y avisa: «Tu status line cambió después de activar; la dejé como está». Desde que el usuario reemplazó el encadenador, el panel muestra el % estimado (AC41) y Opciones dice que el % real dejó de llegar.
  - En los tres casos el panel vuelve al % estimado en 2 s o menos.
  - Con el control activo, el deshacer sigue la regla de AC52.
- AC41 (US8, borde): Sin dato real, se vuelve a la estimación. Check: pasarle al comando un JSON sin `rate_limits` (como antes de la primera respuesta o con API key), o no pasarle nada durante más de 10 min. Pass: el panel muestra el % con la etiqueta «estimado» y, si hubo un dato real antes, su antigüedad. No hay errores.
- AC42 (US8, borde): No escribe sobre un archivo inválido o bloqueado. Check: `settings.json` con JSON roto, y en otro intento el archivo abierto en modo exclusivo por otro proceso. Pass: la app no escribe nada, explica por qué y el hash no cambia.

**US13: responder y aprobar desde Agent Office (las partes que pasen el Paso 0B)**
- AC43 (US13): Activar avisa y solo agrega lo necesario. Check: como AC37, tocando «Activar control desde Agent Office». Pass:
  - Antes de escribir, la app explica qué hooks instala, en qué eventos, qué canal local usa y que nunca gasta tokens, y pide confirmar.
  - Queda el backup propio del control (con un nombre distinto del backup del % real), idéntico al archivo antes de esta activación.
  - Los hooks que el usuario ya tenía para esos eventos quedan intactos.
  - La única diferencia en `settings.json` son los hooks con matcher para `AskUserQuestion` (en `PreToolUse`) y para `PermissionRequest`, según las partes que pasaron el Paso 0B. Son de tipo `command` o `http`, nunca `prompt` ni `agent`.
- AC44 (US13, responder): Una pregunta se contesta desde Agent Office. Check: en un proyecto de prueba con la opción activa, (a) Main hace una pregunta con 3 opciones; elegir la segunda y tocar «Enviar respuesta». (b) Main hace una llamada con 2 preguntas, una de elección única y otra `multiSelect`; elegir una opción en la primera y dos en la segunda, y enviar. Pass:
  - Las preguntas y sus opciones aparecen en el panel en 2 s o menos.
  - Después de enviar, Main sigue con esas respuestas en 2 s o menos (en (b), las dos en la misma respuesta), y el transcript las muestra.
  - La sesión sale de «Te esperan» y no queda en la app de escritorio ningún diálogo que pida acción.
  - Si el Paso 0B mostró que el hook no puede contestar varias preguntas o `multiSelect`, en (b) la tarjeta queda en solo lectura con el motivo «Esta pregunta tiene un formato que no puedo contestar desde acá: contestala en Claude», y (a) sigue funcionando.
- AC45 (US13, aprobar): Un permiso se aprueba o se rechaza desde Agent Office. Check: en modo `default`, una sesión pide permiso para un comando Bash. Usar un comando de 300 caracteres cuya parte final difiera de la inicial. Probar «Aprobar» en una corrida y «Rechazar» en otra. Pass:
  - La tarjeta muestra la herramienta y el comando completo, con scroll si no entra. El recorte a 60 caracteres queda solo en la línea de tiempo.
  - «Aprobar» y «Rechazar» actúan con un solo toque, sin confirmación extra.
  - Aprobar ejecuta el comando; Rechazar hace que no se ejecute y que Main lo vea como rechazado. Cada decisión se aplica en 2 s o menos.
- AC46 (US13, borde): Con Agent Office cerrado, Claude funciona como sin hooks. Check: con la opción activa, cerrar Agent Office y provocar una pregunta y un permiso. Pass: el diálogo de la app de escritorio aparece y funciona, con 1 s o menos de demora extra frente a la misma prueba sin hooks.
- AC47 (US13, borde): Contestar en otro lado no deja nada colgado. Check: con la pregunta visible en Agent Office, (a) contestarla en la app de escritorio; (b) en otra corrida, tocar «Contestar en Claude» en Agent Office. Pass:
  - (a) La pregunta desaparece de Agent Office en 2 s o menos, y 5 s después no queda vivo ningún proceso del hook.
  - (b) El hook se libera sin decidir y la app de escritorio muestra su propio diálogo en 2 s o menos.
- AC48 (US13): Deshacer saca los hooks. Check: con el % real apagado, tocar «Deshacer control» en los casos (a) y (b) de AC40. Pass: los hooks de la app desaparecen, los hooks propios del usuario quedan intactos, el resto del archivo queda como en AC40 (en (a), con el hash del backup del control) y, en las preguntas y permisos siguientes, no se lanza ningún proceso del hook (comprobado en la lista de procesos). Con el % real activo, el deshacer sigue la regla de AC52.
- AC49 (US13): El canal es solo local. Check: con la opción activa, correr AC44 y AC45 con `measure-resources.ps1 -Network` y listar los puertos en escucha de la app con `Get-NetTCPConnection -State Listen`. Además, mandar al canal un mensaje mal formado. Pass:
  - Hay 0 conexiones remotas.
  - Si el canal usa un puerto, escucha solo en 127.0.0.1 o ::1.
  - El mensaje inválido se ignora, no aprueba ni contesta nada y deja una advertencia en el log.

**US15: pregunta o permiso en solo lectura**
- AC50 (US15): Sin control activo, la pregunta se ve y se puede saltar a Claude. Check: con la opción de US13 apagada (o para la parte que falló en el Paso 0B), una sesión hace una pregunta con 3 opciones, otra hace una llamada con 2 preguntas (una `multiSelect`) y una tercera pide un permiso. Pass:
  - El panel muestra, leídas del transcript, las preguntas con sus opciones (indicando cuál es de selección múltiple) y la herramienta con el comando completo, en solo lectura y sin controles para contestar.
  - Cada tarjeta lleva la etiqueta «Solo lectura» y una sola frase con el motivo.
  - «Ir a Claude» trae al frente la ventana de la app de escritorio en 1 s o menos.
  - Si Windows no deja traerla al frente, la app muestra «Abrí Claude: <sesión> te espera», sin error.

**Opciones opt-in (US8 y US13)**
- AC51 (US8, US13): El aviso del README coincide con el de la app. Check: leer el README y el aviso que muestra la app antes de activar cada opción. Pass: el README tiene una sección de opciones opt-in que dice, para cada una, qué escribe en `settings.json`, dónde queda el backup, cómo se deshace, qué canal local usa y que con las opciones apagadas la app es de solo lectura. El aviso de la app dice lo mismo.
- AC52 (US8, US13): Las dos opciones conviven sin romperse. Check: en una copia de prueba con un `settings.json` que ya trae una `statusLine` y un hook propio del usuario, correr dos secuencias: (1) activar % real, activar control, deshacer % real, deshacer control; (2) activar control, activar % real, deshacer control, deshacer % real. Después de cada paso, pasarle al comando de status line el JSON de AC38 y provocar una pregunta. Pass:
  - Después de cada paso, la opción que sigue activa funciona (llega el % real, o la pregunta se puede contestar desde la app), y la `statusLine` y el hook del usuario siguen funcionando.
  - Cada deshacer intermedio quita solo las claves de su opción y no restaura un archivo entero.
  - Cada opción tiene su propio backup, y la segunda activación no pisa el backup de la primera.
  - Al final de cada secuencia, el hash de `settings.json` es igual al del archivo original, antes de cualquier opción.

**Agregados**
- AC53 (US3, US10): El escaneo histórico que calibra el % no traba la app. Check: fixture `history-30d` con 30 días de actividad: 100 transcripts de Main y 300 de subagentes en 6 proyectos, unos 500 MB en total. Abrir la app sobre el fixture y medir con `measure-resources.ps1` durante el escaneo y al terminar. Pass:
  - «Te esperan» y la escena aparecen en el tiempo normal (v2-AC12), sin esperar al escaneo.
  - Mientras escanea, la sección del límite dice «Calculando el % (leyendo el historial)…». El % aparece en 30 s o menos.
  - La memoria privada del host es de 40 MB o menos durante y después del escaneo. Al terminar, la CPU vuelve a los valores de AC29.
- AC54 (US1): Una espera «Terminó» dura lo que debe. Check: escenario `waiting-finished`. La sesión C termina su turno (AC5) y nadie le escribe durante 45 min, con `activeWindowMinutes` en 30. En otra corrida, con el tiempo máximo de «Terminó» en 1 h en la configuración, se espera 61 min. Pass:
  - A los 45 min, C sigue visible en la barra lateral y en «Te esperan», con unos 45:00.
  - Pasado el tiempo máximo de «Terminó» (4 h por defecto, configurable), C sale de «Te esperan» y queda «inactiva». Desde ahí vale `activeWindowMinutes` para mostrarla o no.
  - Si el usuario le escribe antes, sale de la lista como en AC2.
- AC55 (US13): Ni el timeout ni un canal caído traban la sesión. Check: con el control activo, (a) Main hace una pregunta y nadie contesta durante el timeout de control (120 s por defecto, configurable entre 15 y 600 s); (b) con Agent Office abierto, cortar el canal local y provocar una pregunta; (c) volver a habilitar el canal y tocar «Reintentar canal». Pass:
  - (a) A más tardar 2 s después del timeout, el hook se libera sin decidir, la app de escritorio muestra su diálogo y la tarjeta pasa a solo lectura con el motivo «Se venció el tiempo: contestá en Claude».
  - (b) En 2 s o menos el hook se libera sin decidir, la app de escritorio muestra su diálogo y la tarjeta pasa a solo lectura con «El canal con Claude no responde. Contestá en Claude: tu sesión no queda trabada.» y el botón «Reintentar canal».
  - (c) Opciones vuelve a «Activo», y la pregunta siguiente se puede contestar desde la app.
- AC56 (US1): La escena muestra las otras sesiones que esperan. Check: escenario de AC1 con el edificio A seleccionado; después, tocar la píldora de C. Pass:
  - Arriba de la escena hay una píldora por cada otra sesión que espera (B y C), con el título, el motivo en texto y el tiempo.
  - Tocar la píldora de C selecciona C en la barra lateral y en el 3D en 500 ms o menos.
  - Sin esperas en otras sesiones, no hay píldoras.
- AC57 (US1, US3): El panel colapsado deja un resumen. Check: en el escenario de AC1 con `usage-basic`, tocar «Ocultar» en el panel; después, el simulador hace que B deje de esperar; por último, tocar la píldora. Pass:
  - Queda una píldora en la escena con «Panel · 2 te esperan · ~50 % (est.)», o «(real)» si el % es real.
  - En 2 s o menos pasa a «1 te espera».
  - Tocarla vuelve a abrir el panel, y la escena mantiene `fit ok` en los dos estados.
- AC58 (US1, US15): «Ir a Claude» también sirve en las esperas «Terminó». Check: en el escenario de AC1, abrir la tarjeta de C (Terminó). Pass: la tarjeta tiene «Ver qué pasó», que abre la línea de tiempo de C, e «Ir a Claude», que se comporta como en AC50.
- AC59 (US13, US15): Una pregunta que llegó antes de poder contestarla queda en solo lectura. Check: (a) con el control apagado, Main hace una pregunta; activar el control con la pregunta todavía pendiente. (b) Con el control activo y Agent Office cerrado, Main hace una pregunta; abrir Agent Office. (c) En los dos casos, Main hace después otra pregunta. Pass:
  - (a) La tarjeta sigue en solo lectura con «Solo lectura: llegó antes de activar el control. Contestala en Claude; las próximas las contestás acá.».
  - (b) La tarjeta está en solo lectura con un motivo propio de «llegó con la app cerrada».
  - (c) La pregunta nueva se puede contestar desde la app.
- AC60 (US13): Responder es de dos pasos y aprobar, de uno. Check: con el control activo, Main hace una pregunta con 3 opciones; recorrer las opciones con el teclado, elegir una y no enviar durante 10 s; después tocar «Enviar respuesta». En otra corrida, pedir un permiso. Pass:
  - Las opciones son un grupo de elección única (o de casillas, si es `multiSelect`) que se recorre con las flechas.
  - «Enviar respuesta» está deshabilitado hasta que cada pregunta tenga al menos una opción elegida, y el texto de ayuda lo dice.
  - Elegir no manda nada: durante esos 10 s, Main sigue esperando y el transcript no cambia.
  - Al tocar «Enviar respuesta», el botón dice «Enviando…» y se aplica como en AC44.
  - Los permisos se deciden con un solo toque, como en AC45.
- AC61 (US2, US7): Opciones informa sin cambiar nada. Check: abrir Opciones con la configuración por defecto; después, apagar en `config.json` el registro de uso y prender «Pregunta», reiniciar y volver a abrir Opciones; por último, repetir AC11. Pass:
  - Hay tarjetas de solo lectura que muestran el estado de cada motivo de aviso (Pregunta, Terminó, Permiso, alerta de límite) y del registro de uso (prendido o apagado, dónde está y sus últimas marcas, sin contenido).
  - Reflejan los cambios de `config.json` después del reinicio.
  - Después de AC11 aparece la nota neutra «Windows no entregó el último aviso (revisá Configuración > Sistema > Notificaciones)».
  - Ninguna de esas tarjetas tiene controles que cambien un ajuste.
- AC62 (US1–US15): Lo nuevo está en castellano y lo existente no cambia. Check: recorrer todas las pantallas nuevas (panel, pestañas, tarjetas, píldoras, diálogos, notificaciones) y las existentes (barra lateral, botones de cámara, overlay F3). Pass: todo texto nuevo está en castellano rioplatense con voseo. Los rótulos que ya existían (por ejemplo, Buildings, Fit y los campos de F3) siguen en inglés, sin cambios.
- AC63 (US8, US13): Desinstalar no deja la configuración rota. Check: con las dos opciones activas, en una copia de prueba con una `statusLine` y un hook propios del usuario, desinstalar Agent Office con su desinstalador. Después, abrir una sesión, mandar un mensaje y provocar una pregunta. Pass:
  - El desinstalador corre el deshacer de las dos opciones con la regla de AC52. Si el archivo no cambió por fuera, el hash final es igual al original.
  - La `statusLine` del usuario funciona.
  - No queda ningún hook ni ninguna `statusLine` que apunte a un ejecutable de Agent Office.
  - La pregunta aparece en la app de escritorio como sin hooks.

## Constraints

- **Plataforma:** Windows 11, la app de escritorio de Claude Code y el stack actual, Tauri 2 + Three.js (ADR 0001). El core sigue sin depender del renderer (ADR 0003).
- **Tokens:** cero tokens de LLM. Nada de hooks `prompt` o `agent` ni de resúmenes con modelos.
- **Red:** la app no hace conexiones fuera de loopback. Un canal local (loopback, pipe o archivo), cuando exista para US8 o US13, va apagado por defecto y se explica en el README y dentro de la app antes de activarlo («siempre avisado»).
- **Configuración del usuario:** por defecto la app es de solo lectura sobre `~/.claude`. Escribir en `~/.claude/settings.json` solo ocurre con una de dos opciones independientes que el usuario activa: % real (US8) y control (US13). Reglas:
  1. **Aviso:** cada opción se explica antes de activarla, en el README y dentro de la app.
  2. **Backup por opción:** cada opción hace su propio backup, con un nombre distinto, justo antes de escribir. Una activación nunca pisa el backup de la otra.
  3. **Escritura mínima:**
     - El control solo agrega sus hooks y deja intactos los que el usuario ya tenía.
     - El % real agrega la clave `statusLine`. Si ya hay una `statusLine`, la reemplaza por el encadenador de la app, que llama al comando original y devuelve su salida tal cual, para que la barra del usuario siga funcionando. El comando original se guarda en la configuración propia de la app y en el backup de la opción. El aviso previo lo dice.
  4. **Deshacer:**
     - Si la otra opción está apagada y el archivo no cambió desde la activación, se restaura el backup propio byte a byte.
     - Si no, se quitan solo las claves propias. La `statusLine` original solo se restaura si la clave todavía tiene el encadenador; si el usuario la cambió, no se toca y se avisa.
     - Al deshacer la última opción activa, si el archivo queda equivalente al que había antes de cualquier opción, se restaura ese original byte a byte (AC40, AC48 y AC52).
  5. **Desinstalar:** el desinstalador corre el deshacer de las dos opciones (AC63).
  6. **Timeout del control:** mientras el hook espera una decisión de Agent Office, el timeout es de 120 s por defecto, configurable entre 15 y 600 s. Con la app cerrada o el canal caído, el hook se libera sin decidir en 2 s o menos (AC46 y AC55).
- **Secretos:** nunca abrir `*.key`.
- **Recursos:** memoria privada total de 500 MB o menos y del host de 40 MB o menos (ADR 0004). CPU dentro de v1-AC15, también minimizada mientras vigila. Render bajo demanda.
- **Historia:** sin historia propia de largo plazo. Solo los transcripts, más el log de uso sin contenido de US7 (que incluye la hora de la última vez que miraste la app, usada por «nuevo»).
- **Escaneo histórico:** calibrar el % (A1) exige leer los transcripts completos de hasta 30 días, de todos los proyectos y subagentes. Hoy el core lee como máximo los últimos 8 MiB de cada transcript y solo sigue las sesiones activas. Ese escaneo va aparte del seguimiento en vivo, se rehace en cada arranque (no se guarda) y tiene topes: el % sale en 30 s o menos y el host no pasa de 40 MB (AC53).
- **Ubicación de la información:** panel 2D al costado del 3D, más elementos dentro de la escena: el marcador sobre la oficina del jefe, las píldoras de las otras sesiones que esperan y la píldora del panel colapsado. Sin ícono en la bandeja: las notificaciones son de Windows (toast).
- **Cuándo notificar:** siempre que la ventana de Agent Office no tenga el foco: minimizada, visible al costado o tapada (AC9). Con el foco, no se notifica y alcanza con el panel.
- **Idioma:** todo texto nuevo va en castellano rioplatense, con voseo. Los rótulos que ya existen (Buildings, Fit, F3) quedan en inglés (AC62).
- **Accesibilidad:** ningún estado se transmite solo con color (siempre hay texto o un ícono con etiqueta). Las notificaciones son texto legible. El panel se puede colapsar para devolverle espacio al 3D.
- **Legal y privacidad:** ningún dato sale de la máquina. Las notificaciones no llevan prompts, rutas ni comandos, porque Windows las guarda en su centro de notificaciones.
- **Plazo y presupuesto:** no se conocen.

## Assumptions and risks

- A1 (decisión mía): el % del MVP es una estimación. El techo es la mediana de los tokens contados en el tramo de calibración de cada choque que hay en disco. Ese tramo es `[resetsAt − 5 h, hora del rechazo]`, es decir, la ventana real de ese choque hasta el rechazo. Se recalcula al arrancar con el escaneo histórico y no se guarda.
- A2 (decisión mía): «tokens contados» es una fórmula fija sobre `message.usage` (por ejemplo, entrada + caché escrita + salida, con la caché leída ponderada) que elige el architect, deja documentada y usa igual para calibrar y para la ventana actual. El panel también muestra las categorías crudas.
- A3 (decisión mía): la ventana de 5 h es un bloque rodante. Un bloque empieza con la primera actividad (un mensaje de asistente en cualquier proyecto) posterior al fin del bloque anterior, y dura 5 h. La ventana actual es el bloque que contiene «ahora». Si hay un `resetsAt` real (de un choque, o de US8), ese dato fija el bloque: `[resetsAt − 5 h, resetsAt]`.
- A4 (confirmado por el usuario en el Gate 2): la app de escritorio ya avisa cuando Main hace una pregunta, y según su documentación también avisa cuando termina una tarea. Por eso las notificaciones de Agent Office por «Pregunta» y por «Terminó» vienen apagadas (se pueden prender, AC10). Las de «Permiso» y las alertas de límite vienen prendidas. «Terminó» apagado es decisión mía, para no duplicar avisos. Excepción: si el Paso 0B (B1) muestra que Claude no avisa mientras el hook retiene la pregunta, el aviso por «Pregunta» de Agent Office se prende solo mientras el control esté activo (AC10 (c)).
- A5 (decisión mía): las notificaciones muestran el título de la sesión y el motivo, sin el texto de la pregunta, por privacidad.
- A6 (decisión mía): el log de uso de US7 está prendido por defecto, porque sin él no se pueden leer las métricas. Se avisa en el README y se puede apagar.
- A7 (decisión del usuario, Gate 3, F20: «Desde la última vez»): un evento es «nuevo» si ocurrió después de la última vez que miraste la app, es decir, el último momento en que la ventana perdió el foco o se minimizó. Esa hora se guarda en el log de uso, sin contenido, y sobrevive al reinicio. Con el log de uso apagado o vacío, todo lo que hay al abrir cuenta como nuevo.
- A8 (decisión mía): en el MVP, las preguntas se contestan eligiendo opciones: una en las de elección única y una o más en las `multiSelect`. Una llamada puede traer varias preguntas, y se contestan todas juntas con un solo «Enviar respuesta». La opción de texto libre («Otra») no se ofrece desde la app (US16): para usarla, está «Contestar en Claude». Si el Paso 0B muestra que el hook no puede contestar varias preguntas o `multiSelect`, esos casos quedan en solo lectura (AC44).
- A9 (decisión mía): el tamaño de los mensajes entre agentes (US9) se estima a 4 caracteres por token y se marca con «~». En US10, todo sale exacto de `message.usage` (A11). El informe devuelto se toma de los `output_tokens` del último mensaje del subagente.
- A10 (decisión mía): una reescritura de caché tras pausa es una llamada que no lee de caché y vuelve a escribirla después de una pausa más larga que el TTL (5 min en subagentes y 1 h en Main, según la documentación).
- A11 (decisión mía, fórmulas de US10 sobre cada llamada de un agente):
  - **Arranque:** entrada + caché escrita de la primera llamada.
  - **Reescritura tras pausa:** caché escrita de cada llamada que cumple A10.
  - **Crecimiento del contexto:** entrada + caché escrita de las demás llamadas.
  - **Caché leída:** la suma de `cache_read_input_tokens`, ponderada como en A2.
  - **Salida:** la suma de `output_tokens` (incluye el pensamiento). El informe devuelto es una parte de la salida y no se suma aparte.
  - **Total del agente:** la suma de los tokens contados (A2) de todas sus llamadas. Es igual a arranque + reescrituras + crecimiento + caché leída ponderada + salida.
- A12 (decisión mía): una espera «Terminó» deja de contar cuando el usuario le escribe a la sesión, o cuando pasa el tiempo máximo de «Terminó» (4 h por defecto, configurable). Mientras cuenta, la sesión se muestra aunque supere `activeWindowMinutes`. Después pasa a «inactiva» (AC54).
- A13 (decisión mía): el timeout del control es de 120 s por defecto, configurable entre 15 y 600 s. Es un equilibrio: con un timeout corto, la pregunta pasa rápido a Claude si no estás en Agent Office; con uno largo, tenés más tiempo para contestar acá. Si el Paso 0B muestra que el diálogo de escritorio aparece igual mientras el hook espera, el architect puede subir el valor por defecto.
- A14 (decisión del usuario, Gate 3): responder es de dos pasos (elegir y «Enviar respuesta»), porque no hay deshacer. Aprobar o rechazar es de un toque, con el comando completo a la vista (AC45 y AC60).
- R1: la estimación puede fallar. Hay pocos choques en disco, el uso desde claude.ai u otros dispositivos también consume el límite y no aparece en los transcripts, y el peso real de la caché no está documentado. Mitigación: la etiqueta «estimado», el Paso 0A y US8. La métrica de choques sin aviso lo detecta.
- R7: mientras un hook espera la decisión de Agent Office, la app de escritorio podría no mostrar su diálogo (ni su notificación), o mostrarlo y competir con el hook. El Paso 0B lo mide antes de construir. AC46, AC47 y AC55 lo cubren («Contestar en Claude» y el timeout liberan el hook), y si Claude no avisa, AC10 (c) prende el aviso por «Pregunta» mientras el control está activo.
- R10: para que el clic en una notificación devuelva la app con la sesión elegida (AC8), Windows necesita activación de toasts con una identidad de app registrada por el instalador. No está verificado que funcione con la vía de notificaciones que se elija, y en `tauri dev` las notificaciones pueden salir a nombre de otro proceso. Mitigación: el Paso 0C y su plan B (AC8).
- R11: si el usuario borra o mueve a mano el ejecutable de Agent Office sin desinstalar, con una opción activa, la `statusLine` encadenada y los hooks quedan apuntando a algo que no existe. El desinstalador lo cubre (AC63). El caso manual se explica en el README: hay que deshacer las opciones antes de borrar.
- R8: en modo `auto` los permisos son raros, así que «aprobar» se va a usar poco. El valor principal de US13 está en responder preguntas.
- R9: si en algún momento el hook no se libera, una sesión podría quedar trabada hasta el timeout del hook. Mitigación: AC46 y AC47, más un timeout corto que define el architect.
- R2: si el usuario cierra la ventana en vez de minimizarla, no hay notificaciones (no hay bandeja). Se explica en el README.
- R3: el registro de sesiones no está documentado y tiene un bug conocido que deja el estado `busy` pegado. La espera se deriva también del transcript (pregunta pendiente o fin de turno), y v1 ya cubre el fallback.
- R4: lo nativo crece (`/usage`, agent view, recap). La app de escritorio ya avisa las preguntas, así que el valor de Agent Office ahí está en contestar desde la app (US13 y US15), no en el aviso. La métrica de uso diario lo muestra.
- R5: el modo No molestar o Asistente de concentración de Windows puede silenciar las notificaciones. Las ve el panel igual.
- R6: el formato de los transcripts no está documentado. Los tipos desconocidos se ignoran (AC21).

## Open questions

None. Las preguntas del Gate 1 quedaron resueltas en el Gate 2:
- Q1 (resuelta): la app de escritorio sí avisa las preguntas, pero el usuario quiere contestarlas desde Agent Office. Por eso las notificaciones por «Pregunta» vienen apagadas (A4) y US13 entra al MVP.
- Q2 (resuelta): si el Paso 0A pasa, US8 va dentro del MVP.
- Q3 (resuelta): una `statusLine` existente se encadena.
- Q4 (resuelta): la prueba de una semana con un hook `Notification` ya no hace falta. La reemplaza el Paso 0B.

## Build handoff

**Objetivo:** sumarle a Agent Office, sin tokens y sin red, un panel lateral y un marcador en el 3D. Tienen que mostrar qué sesiones esperan al usuario (con notificaciones de Windows aunque esté minimizado), los tokens de la ventana de 5 h con un % estimado y una alerta de umbral, qué consume y por qué causa, el flujo entre agentes y una línea de tiempo por sesión. Con las opciones opt-in (aviso, backup y deshacer), además el % real y la posibilidad de contestar y aprobar desde la app.

**Orden de construcción:**
1. **Pasos 0A, 0B y 0C, antes de construir nada.** El 0A y el 0B los hace el usuario, o el tester con él presente; el 0C lo puede hacer el architect solo. Ver el detalle abajo. Sus resultados deciden qué partes de US8 y US13 entran y cómo se cumple AC8.
2. **Base de solo lectura:** US1, US2, US6, US7 y US15, más los agregados de la UX. Termina cuando pasan AC1 a AC11, AC19 a AC22, AC24, AC50, AC54, AC56 a AC58, AC61 y AC62. AC23 y AC25 se cierran en la fase 3, porque usan las alertas de US4.
3. **Consumo y flujo, también de solo lectura:** US3, US4, US5, US10 y US9, con el escaneo histórico. Termina cuando pasan AC12 a AC18, AC32 a AC36 y AC53.
4. **Mecanismo opt-in común:** el aviso previo, un backup por opción, el deshacer con la regla de Constraints, el encadenado de la `statusLine` y el deshacer en el desinstalador. Sobre ese mecanismo va US8 si pasó el Paso 0A. Termina con AC37 a AC42, AC51 y, si hay una sola opción, AC63.
5. **US13**, solo en las partes que pasaron el Paso 0B. Termina con AC43 a AC49, AC55, AC59, AC60 y, con las dos opciones, AC52 y AC63. Las partes que fallaron quedan cubiertas por US15.
6. **Transversales:** AC26 a AC31 se corren al final de cada fase.

Si fallan los dos Pasos 0, las fases 4 y 5 no existen, y el MVP son las fases 2 y 3 (con US15 como única forma de ver preguntas y permisos).

**Paso 0A, status line (unos 15 min):** en un proyecto descartable, configurar en su `.claude/settings.json` de proyecto una `statusLine` mínima que vuelque su stdin a un archivo. Si la app de escritorio no la toma ahí, usar `~/.claude/settings.json` con un backup previo. Abrir una sesión de ese proyecto en la app de escritorio, mandar un mensaje y revisar el archivo. Después borrar la clave.
- **Pasa** si el archivo aparece con `rate_limits.five_hour.used_percentage` y `resets_at`. Entonces US8 entra al MVP (fase 4).
- **Falla** si el archivo no aparece o aparece sin `rate_limits`. Entonces US8 se descarta y queda la estimación de US3, calibrada con los choques que hay en disco (A1 a A3).

**Paso 0B, viabilidad de responder y aprobar (de 1 a 2 h):** se hace en un proyecto descartable, con hooks en su `.claude/settings.json` de proyecto. Los hooks son scripts mínimos que escriben en un archivo y leen la decisión de otro, sin Agent Office y sin red.
- **B0, los hooks corren en la app de escritorio:** un `PreToolUse` con matcher `AskUserQuestion` que registra cada vez que corre. Pasa si corre una vez por pregunta y ninguna con otras herramientas. Si B0 falla, B1 y B2 fallan.
- **B1, responder:** el hook espera 20 s y devuelve la segunda opción con `updatedInput.answers`. Pasa si Main sigue con esa respuesta, el transcript la muestra y la app de escritorio no deja ningún diálogo que pida acción. Además, hay que anotar tres cosas:
  - si el diálogo de escritorio aparece mientras el hook espera;
  - si la notificación de Claude por la pregunta llega mientras el hook espera (decide AC10 (c));
  - si el hook puede contestar una llamada con 2 preguntas, una de ellas `multiSelect` (decide AC44 (b)).
- **B2, aprobar y rechazar:** en modo `default`, un `PermissionRequest` que espera 30 s y devuelve `allow`, y en otra corrida `deny`. Pasa si la decisión se aplica una sola vez, sin ejecutar dos veces ni trabar la sesión, y el diálogo de escritorio no aparece o se cierra solo.
- **B3, liberar el hook (vale para B1 y B2):**
  - Mientras el hook espera, el usuario contesta en la app de escritorio, si el diálogo aparece. Pasa si la sesión sigue con lo que eligió el usuario y el hook se puede terminar sin romper nada.
  - Un hook que devuelve «sin decisión» en 1 s o menos (el caso de Agent Office cerrado) deja que la app de escritorio funcione como sin hooks.
- **Decisión:**
  - Responder preguntas entra al MVP si pasan B0, B1 y B3.
  - Aprobar permisos entra si pasan B0, B2 y B3.
  - Cada parte que falle queda cubierta por US15. Anotar los resultados en la carpeta de la corrida, para el architect.

**Paso 0C, clic en la notificación (unos 30 min, antes de la fase 2):** con un build instalado (no `tauri dev`), mandar una notificación de Windows por la vía elegida, con un argumento (el id de una sesión), y hacer clic en ella con la app minimizada.
- **Pasa** si la app vuelve al frente y recibe el argumento. AC8 se cumple con la sesión seleccionada.
- **Falla** si el clic no llega a la app o llega sin el argumento. Plan B: el clic solo restaura la app con la pestaña Ahora abierta (AC8). Si ni siquiera restaura la app, frenar y avisar al usuario antes de seguir.

**Restricciones:** todas las de Constraints. Además:
- Siguen valiendo los ADR 0001 a 0006 y los presupuestos de v1 y v2.
- Con las opciones opt-in apagadas, la app no escribe en `~/.claude`, no instala hooks y no abre canales locales.
- Usar una API de notificaciones de Windows (o el primer plugin de Tauri que haga falta) es una decisión del architect, siempre que no agregue red y quede en el README.
- Hay que enmendar el ADR 0002 («no hooks»): la decisión del usuario del 2026-11-18 permite hooks y status line solo como opt-in, con matcher limitado a esos eventos, aviso, backup y deshacer.
- Los hooks son de tipo `command` o `http`. Nunca `prompt` ni `agent`.

**Para el architect del software team:** conservá los IDs AC1 a AC63 tal como están en el plan y en los tests. No los renumeres. Si un criterio no se puede cumplir, marcalo y avisá, en lugar de reescribirlo.

## Revisions

- 2026-11-18, Gate 2 (decisiones del usuario):
  - US9 y US10 pasaron de `should` a `must` y entraron al MVP. US10 suma la cantidad de subagentes y el modelo.
  - US8 pasó a `must`, condicionada al Paso 0A. US13 pasó a `must`, por partes según el nuevo Paso 0B. Se agregaron US15 (respaldo en solo lectura con «Ir a Claude», `must`) y US16 (texto libre, `could`).
  - Una `statusLine` existente se encadena (Constraints, AC39 y AC40).
  - La app de escritorio ya avisa las preguntas: las notificaciones por «Pregunta» y por «Terminó» vienen apagadas. Se editaron en el lugar AC7, AC9, AC10 y AC11 (ahora con el escenario de permiso), y A4 y R4.
  - Se agregaron AC32 a AC51. AC26 a AC29 amplían su alcance a las historias nuevas y a las opciones opt-in.
  - Out of scope: la entrada 10 ahora permite los controles de las opciones opt-in. Se sumaron la 12 (escribir prompts nuevos a una sesión) y la 13 (decisiones en lote o automáticas).
  - Open questions Q1 a Q4 quedaron resueltas. Build handoff ahora tiene un orden de construcción explícito, con el Paso 0A y el Paso 0B.
- 2026-11-18, ciclo de correcciones 1 (`review.md`) y agregados del Gate 3:
  - **F1:** un backup por opción, con la regla de deshacer en Constraints (punto 4). AC37, AC40, AC43 y AC48 editados en el lugar. AC52 nuevo, con las dos secuencias de activar y deshacer.
  - **F2:** encadenar una `statusLine` la reemplaza por el encadenador y guarda el original en la configuración propia de la app (Constraints, punto 3). AC39 editado. AC40 suma el caso (c): el usuario cambió su `statusLine` después de activar.
  - **F3:** A1 define el tramo de calibración y A3 la ventana como bloque rodante. AC12 suma los fixtures del tramo que cruza ventanas y `usage-old-hit`.
  - **F4:** nueva restricción de escaneo histórico, con topes. AC53 nuevo.
  - **F5:** A8 cubre varias preguntas, `multiSelect` y «Otra». AC44 y AC50 editados. B1 anota si el hook puede contestar varias preguntas.
  - **F6:** A12 define el ciclo de vida de «Terminó» (4 h por defecto) y su convivencia con `activeWindowMinutes`. AC54 nuevo.
  - **F7:** B1 anota si llega el aviso de Claude con el hook esperando. A4 y AC10 (c): si no llega, «Pregunta» se prende mientras el control esté activo.
  - **F8:** timeout del control de 120 s por defecto, entre 15 y 600 s (Constraints, punto 6, y A13). AC55 nuevo: timeout vencido y canal caído, con «Reintentar canal».
  - **F9:** regla «se notifica salvo que la ventana tenga el foco» (Constraints). AC9 suma los casos (c), visible sin foco, y (d), tapada.
  - **F10:** la métrica de uso cuenta acciones con intención (3 o más por día) y excluye el foco al arrancar. AC23 y AC25 editados.
  - **F11:** la métrica de esperas suma el P75 de fin de turno y fija qué objetivo vale si «responder» falla en el Paso 0B. AC25 editado.
  - **F12:** fórmulas de US10 en A11. El informe se toma de los `output_tokens` del último mensaje (A9). AC32 fija el denominador y AC34 se reescribió con valores por categoría.
  - **F13:** ver los agregados del Gate 3 más abajo.
  - **F14:** la tarjeta de permiso muestra el comando completo, y el recorte de 60 caracteres queda solo en la línea de tiempo. AC45 editado.
  - **F15:** R10 y Paso 0C (clic en la notificación) antes de la fase 2, con plan B. AC8 editado.
  - **F16:** el desinstalador corre el deshacer (Constraints, punto 5). R11 y AC63 nuevos.
  - **Agregados de la UX aprobados en el Gate 3:**
    - Responder es de dos pasos (elegir y «Enviar respuesta») y aprobar o rechazar, de un toque (A14, AC60).
    - Píldoras en la escena para las otras sesiones que esperan (AC56) y píldora del panel colapsado (AC57).
    - «Ir a Claude» también en las tarjetas «Terminó» (AC58).
    - Una pregunta que llegó antes de activar el control, o con Agent Office cerrado, queda en solo lectura (AC59).
    - Estado «canal caído» (AC55).
    - Tarjetas de solo lectura en Opciones para los avisos y el registro de uso. Out of scope 10 ajustado (AC61).
    - Texto nuevo en castellano y rótulos existentes en inglés (Constraints, AC62).
  - **F20 (decisión del usuario: «Desde la última vez»):** «nuevo» cuenta desde la última vez que miraste la app, con la hora guardada en el log de uso. A7 y AC20 editados.
  - **Hallazgos menores sin corregir:** F17, F18, F19, F21, F22, F23, F24, F25 y F26 de `review.md` siguen abiertos.
  - Fases del Build handoff actualizadas. El architect conserva AC1 a AC63.

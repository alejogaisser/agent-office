# Brief de discovery: Agent Office útil, no solo visual

Fecha: 2026-11-18. Investigación: producto existente, datos locales de `~/.claude` (solo lectura, sin abrir `*.key`) y fuentes web. Este brief no es una spec.

## Problem

Un desarrollador solo corre varias sesiones de Claude Code en paralelo desde la app de escritorio, cada una con subagentes de su plugin. Le pasan tres cosas. No se entera a tiempo de qué sesión lo está esperando. No ve cuánto de su límite de 5 horas consumen sus sesiones, agentes y equipos hasta que choca con él. Y cuando vuelve, le cuesta reconstruir qué hizo cada agente, qué archivos tocó y dónde quedó. Además quiere entender cómo fluye el trabajo entre agentes y dónde se van los tokens. Todo esto tiene que resolverse sin gastar tokens ni usar APIs pagas.

Idea original (paráfrasis del pedido): «Que Agent Office sea útil, no solo visual».

## Users

- **Primario: el propio usuario.** Es un dev solo que corre varias sesiones a la vez en la app de escritorio (hoy hay 5 registradas con `entrypoint: claude-desktop`, 4 en un mismo proyecto) y orquesta equipos de subagentes con `gaisser-agents`.
  - Contrata un producto para: (1) cortar el tiempo muerto en que una sesión espera una respuesta suya; (2) no chocar por sorpresa con el límite de 5 horas y saber qué lo consume; (3) entender en un vistazo qué pasó mientras no estaba.
- **Secundario (supuesto, no es el foco):** otros usuarios del plugin `gaisser-agents` u otros devs con sesiones de Claude Code en Windows. Contratarían lo mismo, sobre todo un monitor de Windows que no toque su configuración. Nada en este brief valida este segmento.

## Today

- **Esperas.** La app de escritorio manda una notificación del sistema cuando una sesión termina una tarea y no la estás mirando. La documentación no dice que avise ante preguntas (`AskUserQuestion`) o permisos. El usuario trabaja casi siempre en modo `auto`, así que hay pocos prompts de permiso; la espera típica es una pregunta de Main o un fin de turno. Hoy lo resuelve revisando sesión por sesión. Agent Office v2 ya muestra el estado `waiting` del registro y atenúa la sala, pero no avisa y hay que estar mirando la ventana.
  - Costo: tiempo muerto. En la sesión de este proyecto, las respuestas a preguntas tardaron entre unos 30 s y 4 min con el usuario presente. No hay medición de las esperas que no notó.
- **Costo y límite.** Se entera del límite cuando choca (`You've hit your session limit · resets 4pm`). En los transcripts que hay en disco aparecen 15 ventanas de 5 horas rechazadas distintas entre principios de septiembre y el 2026-11-18: 6 en los últimos 30 días, una de ellas en esta misma corrida.
  - Paliativos existentes: Claude Code espera solo y retoma la tarea después del reset (hasta 3 veces). `/usage` muestra las barras del plan y qué parte del uso se atribuye a cada subagente, skill o plugin.
  - Costo: trabajo frenado horas. Además, al retomar, los subagentes reescriben su caché completa: en esta corrida el coder retomó tras unas 3 h de pausa y reescribió 313.639 tokens de golpe.
- **Qué pasó mientras no estaba.** Relee el chat de cada sesión. Claude Code ofrece un *session recap* automático (un resumen de 400 caracteres como máximo, que gasta tokens en segundo plano) y `/recap`. Agent Office muestra la herramienta y el archivo actuales en un tooltip, pero no guarda historia: el README dice «Nothing is persisted».
- **Flujo entre agentes.** El usuario no sabe si los agentes se hablan entre sí. Los datos locales muestran un esquema de estrella:
  - Main lanza cada subagente (`Agent`), le manda mensajes (`SendMessage`) y recibe su informe (`SubagentHandback` o `tool_result`).
  - Subagente a subagente casi no ocurre: aparece `SendMessage` desde un subagente en 1 solo archivo.

## Alternatives

| Alternativa | Qué hace bien | Dónde se queda corta | Precio |
|---|---|---|---|
| App de escritorio de Claude (nativa) | Corre las sesiones en paralelo. Notifica al terminar una tarea. Anillo de uso del plan y de contexto. | No documenta avisos ante preguntas o permisos. No muestra subagentes, costo por agente ni historia consolidada. No documenta deep link a una sesión específica (`claude://code/new` solo abre una sesión nueva). | Incluida en el plan |
| `/usage` (nativo) | Barras del plan 5h/7d. Atribución del uso a subagentes, skills, plugins y MCP. Recomendaciones (por ejemplo, «Compact before you step away») con 24h/7d. | Hay que tipearlo en cada sesión. Es una foto, no un tablero en vivo. No muestra el flujo entre agentes. | Incluido |
| Status line con `rate_limits` (nativo) | Única fuente documentada local de `five_hour.used_percentage` y `resets_at` antes de chocar. | Exige un comando en `~/.claude/settings.json`. No está documentado que corra en la app de escritorio. Hay un issue que reporta el campo ausente en una versión. | Incluido |
| Agent view `claude agents` (nativo) | Agrupa sesiones en *Needs input / Working / Completed*. Notifica. Permite responder preguntas de opción. | Es solo para sesiones *background* de la CLI. No lista subagentes. Los permisos exigen hacer *attach*. No documenta soporte en escritorio. | Incluido |
| Session recap y `/recap` (nativo) | Resumen automático al volver. | 400 caracteres como máximo, por sesión, y gasta tokens. | Incluido (consume plan) |
| Auto-continue al límite (nativo) | Retoma solo después del reset. | No previene el choque ni dice qué lo causó. | Incluido |
| Remote Control (nativo) | Seguir y responder una sesión local desde el teléfono o la web. | Pasa por los servidores de Anthropic (choca con «sin red»). Es por sesión, no un tablero. | Incluido |
| Hooks `Notification` / `PermissionRequest` / `PreToolUse` (nativo, hazlo-vos-mismo) | Avisan en `permission_prompt`, `idle_prompt` y `agent_needs_input`. `PermissionRequest` puede devolver `allow`/`deny`. `PreToolUse` puede responder `AskUserQuestion` con `answers`. | Exigen editar `settings.json`. Los `command` hooks lanzan un proceso por evento que coincide. No hay evento de «permiso resuelto» (pedido en issues). | Gratis |
| ccusage (OSS, 19,5k★) | Reportes diarios, de sesión y de bloques de 5 h desde datos locales. Statusline. Modo `--offline`. | Estima el bloque; no conoce el límite real. Es CLI, sin estado de espera ni flujo de agentes. | Gratis (MIT) |
| Claude-Code-Usage-Monitor (OSS) | Terminal en vivo con burn rate. Estima el límite con el P90 de tus bloques previos. | La estimación es heurística. Es solo de uso, sin esperas ni agentes. Hay reportes de que el proyecto se frenó. | Gratis (MIT) |
| claude-control (OSS, macOS, 139★) | Estados Working/Waiting/Idle. Notificaciones. Aprueba o rechaza permisos (vía tmux `send-keys`). | Solo macOS y terminal. Sin tokens ni costo. Usa hooks. | Gratis (MIT) |
| claude-session-monitor (OSS) | Solo lee transcripts: «needs input» por heurística, beep y notificación, tokens por cuenta. | Clic para enfocar solo en macOS. Web en `localhost`. Sin subagentes. | Gratis (MIT) |
| Claude Monitor de whhite (OSS, Windows) | Toast de Windows al entrar en espera. Refresca cada 2 s. | Instala 5 hooks en `settings.json`. Sin tokens ni costo. Requiere Node. | Gratis |
| disler multi-agent observability (OSS, 1,6k★) | Captura 12 eventos de hooks (incluye SubagentStart/Stop y PermissionRequest) en un dashboard en vivo. | Pila pesada (Python, Bun, SQLite, WebSocket, Vue). Sin tokens por agente. Sin aprobar desde el UI. | Gratis |
| OpenTelemetry de Claude Code (nativo) | Métricas de tokens y costo con `agent.name`, `query_source` y `parent_agent_id`. Funciona en la app de escritorio. | Exige variables de entorno y un receptor local (OTLP o Prometheus en `localhost`). Duplica lo que ya está en los transcripts. | Gratis |
| No hacer nada | Cero esfuerzo. Lo nativo ya cubre una parte (notificación al terminar, `/usage`, auto-continue). | Siguen las esperas no vistas, los choques con el límite sin aviso y la reconstrucción manual. | n/a |

## Evidence

Datos locales (leídos 2026-11-18). `~/.claude` = `C:\Users\ASUS\.claude`.

- `evidence`: el usuario corre varias sesiones en paralelo desde la app de escritorio. El registro tiene 5 sesiones vivas, todas con `entrypoint: "claude-desktop"`. Fuente: `~/.claude/sessions/*.json`.
- `evidence`: el límite de 5 h es un dolor recurrente. Hay 53 líneas con `quotaLimits: {status:"rejected", rateLimitType:"five_hour", resetsAt:…}`, que corresponden a 15 ventanas distintas, 6 de ellas en los últimos 30 días. Aparecen en sesiones de Main y en subagentes de 5 proyectos. Fuente: `~/.claude/projects/**/*.jsonl` (grep de `quotaLimits`).
- `evidence`: los transcripts **no** traen el % de límite usado antes de chocar. `quotaLimits` solo aparece en el rechazo (con `status:"rejected"`; no hay ninguna línea con `allowed`). Fuente: la misma búsqueda.
- `evidence`: la fuente documentada del % usado es el JSON que recibe la status line (`rate_limits.five_hour.used_percentage`, `resets_at`, `seven_day`). Solo existe para suscriptores Pro/Max/Ultra y después de la primera respuesta. Fuente: docs statusline.
- `evidence`: cada mensaje de asistente trae `message.model` y `message.usage` con `input_tokens`, `cache_creation_input_tokens`, `cache_read_input_tokens`, `output_tokens`, `output_tokens_details.thinking_tokens`, `cache_creation.ephemeral_5m/1h_input_tokens` y `server_tool_use.web_search_requests`. Con esto se pueden contar tokens por sesión, agente, equipo y modelo sin gastar tokens. Fuente: `…\d43e17a7-…\subagents\agent-a788007c9cc07c4e3.jsonl`.
- `evidence` (formato no documentado): hay líneas `type:"cost-state"` esporádicas (no en cada turno) con `totalCostUSD`, `totalLinesAdded/Removed`, `totalAPIDuration` y `modelUsage` por modelo con `costUSD`. Ejemplo: `totalCostUSD` 2,43, con 2,16 M tokens leídos de caché y 48,9 k de salida en Opus. Fuente: `~/.claude/projects/C--dev-Agents/d43e17a7-….jsonl:166`.
- `evidence`: para suscriptores, la cifra en USD **no** es relevante para la facturación; lo que cuenta es el % del plan. Fuente: docs costs.
- `evidence`: cada subagente terminado deja un `attachment` `task-notification` con `usage: {totalTokens, toolUses, durationMs}`. Ejemplos de esta corrida: entre 96.006 y 316.491 tokens por subagente. Fuente: `…\d43e17a7-….jsonl` líneas 150 a 731.
- `assumption`: `totalTokens` es el tamaño del contexto final del subagente, no un acumulado de todas sus llamadas. Lo sugiere que el contexto final del coder (unos 314 k) se parece al valor reportado. No está documentado.
- `evidence`: el `.meta.json` de cada subagente trae `agentType`, `description`, `toolUseId`, `spawnDepth`, `requestShape:"background"`, `pickedEffortModel` y `finished`. Fuente: `…\d43e17a7-…\subagents\agent-add38b3413cd9c71d.meta.json`.
- `evidence`: arrancar un subagente cuesta unos 9,5 k a 13,7 k tokens antes de trabajar (la primera llamada escribe en caché sistema, herramientas, definición y prompt), y su contexto crece hasta 100 k a 300 k o más. Fuente: primeras y últimas líneas `usage` de `agent-a788007c9cc07c4e3`, `agent-add38b3413cd9c71d` y `agent-a2de8d464991100be`.
- `evidence`: una pausa larga es cara. El coder v2 quedó parado de 17:44 a 20:40 UTC y su primera llamada al volver tuvo `cache_read_input_tokens:0` y `cache_creation_input_tokens:313639`. La documentación lo explica: los subagentes usan un TTL de caché de 5 min aun con suscripción, y el chat principal usa 1 h. Fuentes: `agent-add38b3413cd9c71d.jsonl:614-618` y docs prompt-caching.
- `evidence`: los mensajes entre agentes son chicos comparados con el contexto de cada agente. Las cifras salen de contar caracteres; a unos 4 caracteres por token, es una aproximación.
  - Informes de `SubagentHandback`: 346 líneas. El 82 % tiene 2.000 caracteres o más (unos 500 tokens), el 25 % 6.000 o más (unos 1.500) y el 2 % 12.000 o más (unos 3.000).
  - Prompts de `Agent` en la muestra con este orden de campos (94 de 305 líneas): el 70 % tiene 2.000 caracteres o más y solo 1 llega a 6.000.
  - Fuente: grep sobre `~/.claude/projects/**`.
- `signal`: por lo anterior, «comunicación concisa entre agentes» ahorra poco de forma directa. Los informes se releen desde la caché en cada turno siguiente de Main, a 0,1× del precio de input. Las palancas grandes parecen otras: cantidad de turnos y crecimiento del contexto de cada subagente, fallos de caché tras pausas, cantidad de subagentes lanzados y modelo y esfuerzo usados.
- `evidence`: el flujo es en estrella con Main en el centro. `SendMessage` aparece en 186 líneas de 27 archivos; desde subagentes, solo en 1 archivo. `Agent` aparece en 305 líneas de 33 archivos. Los conteos son cotas superiores, porque algunos tool_use aparecen repetidos. Fuente: grep sobre `~/.claude/projects/**`.
- `evidence`: el usuario trabaja casi siempre en modo `auto`. `"permissionMode":"auto"` aparece en 1.309 líneas de 86 archivos; `default`/`acceptEdits`/`plan`/`bypassPermissions` aparecen en 17 líneas de 2 archivos. Por eso los prompts de permiso deberían ser raros. Fuente: grep sobre `~/.claude/projects/**`.
- `evidence`: Main le pregunta seguido al usuario. Hay 189 líneas con `"name":"AskUserQuestion"` en 84 transcripts. Fuente: grep sobre `~/.claude/projects/**`.
- `signal`: en esta corrida, las respuestas a preguntas tardaron unos 81 s, 3,6 min, 4,2 min, 31 s y 3 min (5 casos). Es una muestra chica, con el usuario presente, y mezcla tiempo de pensar con tiempo de no haberlo visto. Fuente: timestamps de `…\d43e17a7-….jsonl`.
- `evidence`: también hay datos de «qué pasó» en los transcripts.
  - `tool_use` de Edit y Write con `file_path`, Bash con `command`, y el texto del informe del subagente.
  - Errores de API (`system/api_error`, `error:"rate_limit"`), `compact_boundary` y timestamps en cada línea.
  - Fuentes: plan v1 (`docs/plan.md`, sección Context) y grep de subtypes.
- `evidence`: los transcripts se borran a los 30 días por defecto (`cleanupPeriodDays`). Una historia de costo más larga obligaría a guardar algo propio. Fuente: docs costs (`/insights`).
- `evidence`: el registro de sesiones trae `status` (`busy`/`idle`/`waiting`), `statusUpdatedAt`, `peerFeatures:["notify_idle",…]` y `messagingSocketPath` (named pipe de mensajería entre sesiones). En Windows, la conexión al pipe exige un token de autenticación (`CLAUDE_CODE_MESSAGING_TOKEN`, que solo reciben los procesos hijos de la sesión, es decir hooks y comandos Bash), y un mensaje entregado cuenta como uso y no puede aprobar permisos. Fuentes: `~/.claude/sessions/33340.json` y docs cross-session-messaging.
- `evidence`: ADR 0002 descartó los hooks porque «spawn a process per tool call» y porque tocan la config del usuario. Hay un matiz: un hook con matcher solo en `PermissionRequest` o `Notification` corre solo cuando ese evento ocurre, no en cada herramienta. Fuentes: `C:\dev\agent-office\docs\decisions\0002-transcripts-only-data-source.md` y docs hooks.
- `evidence`: aprobar sin tokens es posible por mecanismos documentados, pero todos exigen escribir en `~/.claude/settings.json`, lo que es una decisión nueva del usuario.
  - `PermissionRequest` (hooks `command` o `http`) devuelve `decision.behavior: allow|deny`, con un timeout por defecto de 600 s.
  - `PreToolUse` puede contestar `AskUserQuestion` con `updatedInput.answers`.
  - Los hooks `prompt` y `agent` sí gastan tokens y quedan descartados.
  - Fuentes: docs hooks y docs desktop (sugiere un `PermissionRequest` hook para «allow or deny requests in your place»).
- `assumption`: mientras un hook `PermissionRequest` o `PreToolUse` bloquea esperando la decisión desde Agent Office, la app de escritorio sigue mostrando su propio diálogo y gana el que responde primero. No está documentado para escritorio; podría pasar lo contrario: que el diálogo no aparezca hasta que el hook termine.
- `evidence`: el canal app↔hook tiene dos caminos y ambos tocan las garantías actuales. Un hook `http` exige que la app escuche en loopback (hoy: «no network connections»). Un hook `command` exige un helper que lance Claude Code e intercambie archivos con la app (hoy la app solo escribe su config y su log). Fuentes: `C:\dev\agent-office\README.md` y docs hooks.
- `evidence`: el problema de «no sé qué sesión me espera» es común. Hay un issue en anthropics/claude-code: «The bottleneck increasingly isn't Claude's speed — it's the human noticing that Claude is waiting» (5 👍, cerrado como *not planned* por inactividad). Hay al menos 5 monitores de terceros que atacan el mismo problema. Fuentes: issue #36885, claude-control, claude-session-monitor, Claude Monitor de whhite y la nota de eyes-on-claude-code.
- `evidence`: el drenaje rápido de límites es un problema público. Anthropic admitió que «people are hitting usage limits in Claude Code way faster than expected» (marzo y abril de 2026). La caché de 5 min encarece volver de una pausa. Fuente: DEVCLASS.
- `signal`: en usuarios de terceros, correr subagentes en paralelo agota el límite más rápido (Pro: unos 15 min contra unos 30 en secuencial; Max $100: unos 75 min). Es un solo autor, de julio de 2025. Fuente: dev.to (onlineeric).
- `evidence`: Anthropic ya cubre parte del costo de forma nativa. `/usage` atribuye el uso a subagentes, skills y plugins y recomienda cambios. Las docs listan como causas de consumo el contexto largo, los fallos de caché, los subagentes y la compactación. Fuente: docs costs.
- `assumption`: el usuario va a mirar Agent Office a diario como tablero principal si resuelve las esperas y el límite. Está declarado en la intake y sin dato de uso real.
- `assumption`: la visualización 3D sigue siendo deseada como contenedor. El pedido es sumarle utilidad, no reemplazarla por un tablero 2D. Es mi lectura del pedido.
- `assumption`: la app de escritorio no avisa cuando una sesión hace una pregunta (`AskUserQuestion`); solo avisa al terminar. La documentación no lo menciona en ningún sentido.
- `assumption`: la app de escritorio ejecuta los hooks y la `statusLine` de `~/.claude/settings.json`. Los docs de escritorio mencionan hooks para SSH remoto, pero no confirman la `statusLine`.

## Riskiest assumptions

1. **Se puede mostrar «cuánto límite me queda» sin tokens y sin red.** Si es falso, el pilar de costo se reduce a estimaciones.
   - Riesgo: los transcripts solo informan el rechazo, y la fuente documentada (`rate_limits` de la status line) puede no correr en escritorio.
   - Prueba más barata: con el permiso del usuario, configurar una `statusLine` mínima que vuelque su stdin a un archivo, abrir una sesión en la app de escritorio, mandar un mensaje y ver si el archivo aparece con `rate_limits.five_hour`. Son unos 15 minutos y se deshace borrando la clave.
   - Plan B sin tocar settings: calibrar el techo con los tokens consumidos en las 5 h previas a cada uno de los 15 rechazos registrados, en lugar de un P90 genérico.
2. **Aprobar o responder desde Agent Office ahorra tiempo de verdad.** Si es falso, «control» no vale la complejidad ni ceder la garantía de solo lectura.
   - Riesgo: en modo `auto` los prompts de permiso son raros; lo que bloquea son las preguntas y los fines de turno. Además, un hook que bloquea podría ocultar o retrasar el diálogo de la app.
   - Prueba: durante una semana, un hook `Notification` que solo registra en un archivo, para contar `permission_prompt`, `idle_prompt` y `agent_needs_input` por día. En paralelo, en un proyecto descartable, un `PermissionRequest` hook que espera 30 s, para ver qué hace el diálogo de escritorio.
3. **El dolor de las esperas no lo cubre ya la app de escritorio.** Si es falso, el valor de las alertas cae a casi cero.
   - Riesgo: la app ya notifica cuando una tarea termina; no sabemos si notifica preguntas.
   - Prueba: que el usuario provoque un `AskUserQuestion` con la ventana de esa sesión en segundo plano y observe si llega una notificación. Además, que anote durante 3 días cada vez que encuentre una sesión que lo esperaba hacía más de 5 min.
4. **Hacer más concisa la comunicación entre agentes ahorra tokens en forma apreciable.** Si es falso, ese ángulo debería reorientarse a «dónde se van los tokens» (contexto, caché, cantidad de spawns, modelo).
   - Riesgo: los datos locales indican que prompts e informes pesan de 0,5 k a 3 k tokens frente a los 100 k a 300 k de contexto de cada subagente.
   - Prueba (cero tokens): extender `office-dump` o un script para que, sobre los últimos 30 días, reparta los tokens por agente en arranque, crecimiento del contexto, reescrituras de caché e informes devueltos a Main.
5. **Va a ser el tablero diario y no un juguete.** Si es falso, el esfuerzo se va a la vista 3D sin uso real.
   - Riesgo: las herramientas nativas crecen rápido (`/usage` con atribución, agent view, recap, auto-continue) y podrían volverse «suficientes».
   - Prueba: una semana con la versión actual más un contador local de aperturas y de foco de la ventana (en el log propio de la app), y una entrevista corta al final: ¿qué miraste?, ¿qué te faltó?

## Opportunity

1. **Esperas:** un estado «te espera» por sesión, derivado sin tokens del registro y del transcript (`waiting`, `AskUserQuestion` pendiente, `end_turn`), con un toast de Windows y un contador en la bandeja. Hoy nadie lo resuelve en Windows sin instalar hooks, y lo nativo es de CLI o solo avisa al terminar.
2. **Límite y costo:** tokens por sesión, agente, equipo y modelo; reescrituras de caché tras pausas; la hora y la causa de cada choque, y una alerta de «vas rumbo al límite». El % real depende del riesgo 1. Este es el dolor mejor evidenciado: 15 choques registrados.
3. **Qué pasó:** una línea de tiempo por sesión, armada con datos que ya existen y sin tokens: spawns y handbacks, archivos editados, comandos, errores y límites, y la primera línea de cada informe. Es un *recap* gratis y más completo que el nativo, que tiene 400 caracteres y cuesta tokens.
4. **Flujo y ahorro:** mostrar que el flujo es en estrella con Main en el centro (los agentes no se hablan entre ellos) y dónde se van los tokens. La evidencia apunta a contexto, caché, cantidad de spawns y modelo, más que a la longitud de los mensajes entre agentes.
5. **Control y por qué podría no valer:** aprobar o responder solo es posible con hooks en `settings.json`, más un canal hook↔app, lo que rompe la garantía actual de solo lectura y sin red. Con modo `auto`, el beneficio puede ser chico. Si lo nativo absorbe 1 a 3, el diferencial queda en la vista unificada en Windows y en el análisis local de tokens.

## Sources

Archivos locales (leídos 2026-11-18):
- `C:\dev\agent-office\README.md`
- `C:\dev\agent-office\docs\plan.md`
- `C:\dev\agent-office\docs\plan-v2.md`
- `C:\dev\agent-office\docs\decisions\0002-transcripts-only-data-source.md`
- `C:\dev\agent-office\docs\decisions\0004-ram-budget-250mb.md`
- `C:\dev\agent-office\docs\decisions\0006-building-floor-room-model.md`
- `C:\Users\ASUS\.claude\sessions\*.json` (5 archivos; los `*.key` no se abrieron)
- `C:\Users\ASUS\.claude\projects\**\*.jsonl` (solo búsquedas por nombre de campo y conteos; el contenido no se copió)
- `C:\Users\ASUS\.claude\projects\C--dev-Agents\d43e17a7-2962-4d44-ba4c-12085d86de49.jsonl` y su carpeta `subagents\` (`agent-a788007c9cc07c4e3`, `agent-add38b3413cd9c71d`, `agent-a2de8d464991100be`, `.jsonl` y `.meta.json`)
- `C:\Users\ASUS\.claude\projects\C--dev-Agents\c8b089e4-546e-4748-bb88-c728dcd68871\subagents\agent-afc74161309b31a56.meta.json`
- `C:\Users\ASUS\.claude\cache\` (solo el listado; no hay snapshot de uso del plan)

Web (leídas 2026-11-18):
- https://code.claude.com/docs/en/hooks
- https://code.claude.com/docs/en/statusline
- https://code.claude.com/docs/en/costs
- https://code.claude.com/docs/en/desktop
- https://code.claude.com/docs/en/prompt-caching
- https://code.claude.com/docs/en/agent-view
- https://code.claude.com/docs/en/interactive-mode
- https://code.claude.com/docs/en/monitoring-usage
- https://code.claude.com/docs/en/cross-session-messaging
- https://code.claude.com/docs/en/remote-control
- https://github.com/anthropics/claude-code/issues/36885
- https://github.com/sverrirsig/claude-control
- https://github.com/imabhinavagarwal/claude-session-monitor
- https://claudemonitor.whhite.com/
- https://github.com/ryoppippi/ccusage
- https://github.com/disler/claude-code-hooks-multi-agent-observability
- https://www.devclass.com/ai-ml/2026/04/01/anthropic-admits-claude-code-users-hitting-usage-limits-way-faster-than-expected/5213575
- https://dev.to/onlineeric/claude-code-sub-agents-burn-out-your-tokens-4cd8

Solo resumen de buscador, sin abrir la página (peso `signal`, 2026-11-18):
- https://github.com/Maciek-roboblog/Claude-Code-Usage-Monitor
- https://dev.to/joe_re/i-built-a-desktop-app-to-supercharge-my-tmux-claude-code-workflow-521m
- https://claudeissues.com/issue/64170-feature-add-permissionresolved-hook-event
- https://claudeissues.com/issue/45133-rate-limits-seven-day-five-hour-missing-from-statusline-json-input
- https://claudeissues.com/issue/66762-ultracode-mode-can-silently-consume-an-entire-5-hour-usage-window-on-subagent-fl
- https://macrumors.com/2026/03/26/claude-code-users-rapid-rate-limit-drain-bug

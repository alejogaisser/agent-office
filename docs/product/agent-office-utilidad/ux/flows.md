# UX de Agent Office útil: flujos, pantallas y prototipo

Spec de origen: `spec.md` (con sus Revisions del Gate 2). Prototipo clickeable: `ux/prototype.html` (un solo archivo, sin red, sin datos reales; el 3D es una maqueta estática y todo el foco está en el panel nuevo y en los marcadores dentro de la escena).

La app ya existe: este diseño **extiende** la ventana actual (barra lateral de edificios a la izquierda, oficina isométrica al centro, botones de rotar/Fit abajo a la derecha, tema oscuro, tipografía Segoe UI 13 px). No reemplaza nada. Se agregan tres cosas: un **panel 2D a la derecha**, **marcadores dentro de la escena** y **textos de estado en la barra lateral**. Todo es HTML/CSS/TS plano, sin librerías de UI (restricción de RAM/CPU).

## Cómo recorrer el prototipo

Abrir `ux/prototype.html` en el navegador. Arriba hay una barra violeta, «Simulador de escenarios», que **no es parte del producto**: sirve para provocar cada flujo y cada error (una pregunta nueva, dos preguntas con una `multiSelect`, un permiso con un comando de 300 caracteres, una ráfaga, minimizar la ventana, dejarla visible sin foco o tapada, cerrarla, subir el uso a 85 %, un choque de límite, el escaneo del historial, un `settings.json` roto o bloqueado, que el canal del hook no responda o venza su timeout, que fallen las pruebas 0A, 0B o 0C, etc.). Siempre que la ventana no tenga el foco (minimizada, visible sin foco o tapada) el «escritorio de Windows» simulado muestra las notificaciones; tocarlas restaura la app con la sesión elegida. A ancho de teléfono la barra lateral se abre con el botón «Edificios» y el panel es una hoja inferior que se abre con la píldora «Panel».

El estado inicial reproduce AC1: la sesión «Agent Office utilidad» (B) tiene una pregunta sin responder desde hace 1:30, «Docs del plugin» (C) terminó su turno hace 0:30 y «Migración API clientes» (A) sigue trabajando y no aparece en «Te esperan». Durante los primeros 5 s la tarjeta del límite dice «Calculando el % (leyendo el historial)…» mientras «Te esperan» y la escena ya funcionan (AC53). Los eventos de la línea de tiempo posteriores a las 14:05 (la última vez que «miraste» la app) salen marcados «nuevo».

Revisión del ciclo 1 (spec con AC52 a AC63 y revisión de la UX): backups separados por opción, `statusLine` reemplazada por el encadenador, varias preguntas y `multiSelect`, comando completo en permisos, aviso siempre que la ventana no tenga foco, «Terminó» que vence a las 4 h, «nuevo» desde la última vez, timeout del control de 120 s y fallback del Paso 0C. Los cambios están anotados en `## Revisions` al final.

## Flows

Convención: **E** = error o borde. Cada flujo tiene al menos un camino de error. Casi todos se pueden provocar en el prototipo con el simulador; los pocos que no (umbral inválido en `config.json`, líneas ilegibles del transcript) están solo descritos acá porque no tienen pantalla propia.

### F1. Ver quién te espera y ir a esa sesión (US1; AC1 a AC6)

Entrada: abrir Agent Office o mirarlo de reojo (ventana lateral).
1. La barra lateral muestra, bajo cada edificio, el estado en **texto** («trabajando», «inactiva», «te espera 1:30 · Pregunta»). El 3D muestra, sobre la oficina del jefe del piso seleccionado, un marcador con ícono y texto («? Pregunta · 1:30»). Si otras sesiones esperan, aparecen como píldoras arriba a la izquierda de la escena.
2. El panel abre en la pestaña **Ahora**. La sección «Te esperan (N)» lista solo las sesiones que esperan, de la que más espera a la que menos, cada una con motivo (Pregunta, Permiso, Terminó) y tiempo m:ss que corre.
3. La primera tarjeta con pregunta o permiso viene expandida (ver F15, F12, F13). Una tarjeta «Terminó» muestra «Ver qué pasó» e «Ir a Claude» (AC58) y aclara: «Sale de esta lista cuando le escribas o a las 4 h (se cambia en config.json); después queda «inactiva».»
4. Tocar el encabezado de una tarjeta, una píldora de la escena o el marcador **selecciona** esa sesión en la barra lateral y en el 3D (AC4) y expande su tarjeta.
5. Cuando el usuario le contesta a esa sesión (en Agent Office o en Claude), la tarjeta desaparece en 2 s o menos, el marcador se va y aparece, si corresponde, una confirmación breve (AC2).
- **E1, nadie espera:** la sección muestra «Nadie te espera» con la ayuda «Cuando una sesión necesite algo tuyo, aparece acá y como marcador en el 3D». El panel y el 3D no muestran marcadores.
- **E2, sesión cerrada mientras esperaba:** el punto de la barra lateral pasa a rojo con el texto «cerrada», la tarjeta sale de «Te esperan» y, pasada la gracia de cierre, la fila sale de la barra (AC6).
- **E3, fin de turno con subagentes trabajando (AC5):** no hay tarjeta ni aviso. La fila de la barra sigue como «trabajando». Aparece «Terminó» recién cuando Main cierra del todo.
- **E4, panel oculto (AC57):** si el usuario ocultó el panel, la píldora de la escena sigue diciendo «Panel · 2 te esperan · ~50 % (est.)» (o «(real)», o «calculando el %…» mientras escanea), se actualiza en 2 s y la reabre al tocarla.
- **E5, «Terminó» que lleva mucho (AC54):** a los 45 min la sesión C sigue en la barra lateral y en «Te esperan» con 45:00, aunque supere los 30 min de actividad (el reloj pasa a h:mm:ss desde la hora). Al vencer el tiempo máximo de «Terminó» (4 h por defecto) sale de la lista y queda «inactiva».
- **E6, píldoras (AC56):** con el edificio A seleccionado, las píldoras de B y C (título, motivo en texto y tiempo) están arriba a la izquierda de la escena; tocar una selecciona esa sesión. Sin esperas en otras sesiones no hay píldoras.

### F2. Enterarse con la ventana minimizada (US2, US7; AC7 a AC11)

Entrada: Agent Office abierto pero **sin foco** (minimizado, visible al costado con el foco en otra app, o tapado por otra ventana), una sesión pasa a esperar. La regla es «se notifica siempre que la ventana no tenga el foco» (AC9).
1. Windows muestra una notificación con el título de la sesión y el motivo («Agent Office utilidad · Te espera · Permiso»). Nunca incluye el texto de la pregunta, rutas ni comandos.
2. Se anota una marca en el registro de uso (hora, tipo, id de sesión).
3. El usuario toca la notificación: Agent Office se restaura al frente con esa sesión seleccionada y la pestaña Ahora abierta (AC8). Se anota la marca de clic y de foco.
- **E1, ventana con foco:** no hay notificación; el panel y el marcador se actualizan igual (AC9a). Con la ventana visible sin foco o tapada, sí hay una (AC9c y AC9d); la barra de título muestra «sin foco» en el primer caso.
- **E1b, Paso 0C falló:** el clic en el aviso no puede elegir la sesión. Agent Office se restaura igual al frente, con la pestaña Ahora abierta y la sesión que más espera primera en «Te esperan», sin seleccionar nada (AC8).
- **E1c, alerta de límite:** el clic en el aviso del umbral restaura la app y baja el panel a Ahora > «Límite de 5 h» (no hay sesión que elegir).
- **E2, ráfaga:** tres sesiones piden permiso en pocos segundos: una sola notificación, «3 sesiones te esperan». Al tocarla se abre la que más espera (AC9b).
- **E3, motivo apagado:** «Pregunta» y «Terminó» vienen apagados porque Claude ya los avisa; la sesión aparece igual en «Te esperan». Se prenden en `config.json` (AC10). El panel muestra el estado de cada aviso en Opciones, en solo lectura (AC61).
- **E3b, «Pregunta» se prende sola (AC10c):** si el Paso 0B (B1) mostró que Claude no avisa mientras el hook retiene la pregunta, con el control activo Opciones dice «Pregunta: prendido solo, mientras el control esté activo» y hay un aviso por cada pregunta. Al deshacer el control vuelve a «apagado».
- **E4, Windows bloquea las notificaciones:** la app no muestra error ni se cierra. La sesión se ve en «Te esperan» y Opciones > Avisos de Windows agrega una nota neutra: «Windows no entregó el último aviso (revisá Configuración > Sistema > Notificaciones)» (AC11).
- **E5, ventana cerrada:** no hay avisos. Esto se explica antes, en Opciones («Avisan siempre que la ventana no tenga el foco… Si la cerrás, Agent Office se cierra y no avisa») y en el README. Ver F12 E4 para qué pasa con una pregunta que llega con la app cerrada.

### F3. Ver cuánto límite llevo y recibir la alerta (US3, US4; AC12 a AC16)

Entrada: pestaña Ahora, sección «Límite de 5 h».
1. La tarjeta muestra los tokens de la ventana («600 k tokens en esta ventana»), una barra con la marca del umbral al 80 %, «~50 % (estimado) · se libera ~16:02 (estimado)» y el desplegable «¿Cómo se calcula?» (techo = mediana de las 5 h previas a cada choque registrado; avisa que el uso desde claude.ai u otros dispositivos no aparece).
2. Al cruzar el umbral la tarjeta se resalta con borde ámbar, barra rayada y el texto «Alerta: pasaste el 80 %. Pausá, compactá o cerrá sesiones antes de chocar.». Si la app está minimizada, hay **una** notificación («Llevás ~80 % del límite de 5 h (estimado) · se libera ~16:02»). No hay más hasta la ventana siguiente.
3. «Ver quién consume» lleva a F4.
- **E0, escaneando el historial (AC53):** al arrancar, mientras se lee el historial, la sección muestra los tokens y «Calculando el % (leyendo el historial)…» («Tarda hasta 30 s. «Te esperan» y la escena no esperan al cálculo»). No hay barra, % ni alertas hasta que termina. La píldora del panel colapsado dice «calculando el %…».
- **E1, sin choques registrados (AC14):** se muestran los tokens y el texto «Sin choques registrados para estimar el %…». No hay barra ni %, y no hay alertas.
- **E2, choque en la ventana actual (AC13):** «Límite alcanzado · se libera 16:00», sin la palabra «estimado».
- **E3, umbral inválido en `config.json`:** se usa 80 %; el aviso queda solo en el log (AC16, sin UI). La tarjeta siempre rotula la marca como «umbral 80 %» con el valor vigente.

### F4. Ver qué sesiones y agentes consumen más (US5, US10; AC17, AC18, AC35)

Entrada: pestaña **Consumo**, o «Ver quién consume» desde F3.
1. «Quién consume en esta ventana» muestra el total de la ventana y dos rankings: las 5 sesiones y los 5 agentes (rol + título de la sesión) que más consumieron, con tokens, % del total y barra. Al pie: «+2 sesiones», «+10 agentes» (los datos del prototipo difieren de AC17; Main cuenta como agente y dos agentes con el mismo rol en una sesión se distinguen por el título de la sesión, no por un sufijo).
2. «Dónde se van los tokens» resume la ventana: subagentes lanzados, arranque, crecimiento del contexto, reescrituras de caché tras pausas (resaltadas), informes devueltos a Main, total y tokens por modelo.
3. El enlace «Ver el desglose por agente de la sesión seleccionada» lleva a Sesión > Tokens (F7).
- **E1, sesión ya cerrada:** su consumo cuenta en el total y, si entra en el ranking, se rotula «(cerrada)».
- **E2, sin choques (AC14):** el consumo se muestra igual en tokens; solo faltan los % de límite.

### F5. Retomar sin releer el chat (US6; AC19 a AC22)

Entrada: pestaña **Sesión** > «Qué pasó» (también «Ver qué pasó» en una tarjeta «Terminó»).
1. Línea de tiempo de la sesión seleccionada, de lo más nuevo a lo más viejo. Cada entrada: hora HH:MM, agente, etiqueta de tipo (Lanzó, Informe, Archivo, Comando, Error, Límite, Compactó) y resumen. Informe: primera línea, 120 caracteres como máximo. Archivo: ruta relativa. Comando: primeros 60 caracteres.
2. Lo que pasó **desde la última vez que miraste la app** lleva la marca «nuevo» (decisión del usuario, «Desde la última vez»; A7 y AC20). «Última vez» es el último momento en que la ventana perdió el foco o se minimizó. Arriba de la lista una línea lo dice: ««Nuevo» = desde la última vez que miraste la app (14:05).» y en Opciones > Registro de uso aparece esa hora exacta, que sobrevive al reinicio porque sale del registro de uso.
3. Al pie: «Se arma en vivo desde los transcripts. Nada de esto se guarda en disco.» (AC22).
- **E0, registro de uso apagado (AC20c):** no hay hora de referencia: todo lo que hay al abrir lleva «nuevo» y la línea de arriba lo explica («el registro de uso está apagado y no sé cuándo miraste por última vez»). Sin error.
- **E1, transcript muy grande (AC21):** arriba, «Historia parcial: desde 10:31…», sin error.
- **E2, sesión sin eventos:** «Todavía no pasó nada en esta sesión».
- **E3, líneas ilegibles:** se ignoran en silencio; la línea de tiempo sigue (sin UI).

### F6. Ver cómo fluye el trabajo entre agentes (US9; AC32, AC33)

Entrada: Sesión > «Flujo».
1. Una frase resume: «Main está en el centro: casi todo pasa por él, salvo 1 mensaje directo entre subagentes (resaltado).»
2. Un diagrama en estrella: Main arriba; los subagentes debajo; flechas de lanzamiento («lanza ~1 k») y de informe (línea punteada, «informe ~1,5 k»). Un mensaje de subagente a subagente se dibuja en naranja, más grueso, con la etiqueta «subagente a subagente» (texto, no solo color). Un subagente lanzado por otro cuelga de su padre.
3. «Mensajes entre agentes: ~9 k tokens, 2,5 % de los tokens de la sesión», con la aclaración de que es estimado a 4 caracteres por token.
4. Una tabla repite todo en texto (de → a, tipo, tamaño): es la versión accesible del diagrama.
- **E1, sin subagentes:** «Main no lanzó subagentes».
- **E2, informe pendiente:** el nodo lleva borde punteado y «informe pendiente»; en la tabla el tamaño dice «pendiente».

### F7. Entender dónde se van los tokens de una sesión (US10; AC34 a AC36)

Entrada: Sesión > «Tokens».
1. Arriba: «4 subagentes lanzados · arranque total 46.000 tokens» y tokens por modelo.
2. Una tarjeta plegable por agente, de mayor a menor consumo. Cada una tiene una barra apilada con leyenda de texto y un desglose: arranque, crecimiento del contexto, reescritura de caché tras pausa (con la hora y los minutos parado), informe devuelto a Main (~) y **Total**, que es la suma exacta de las partes. El agente con reescritura viene abierto y aclara «Los subagentes guardan su caché 5 min…».
- **E1, agente sin datos (AC36b):** la fila dice «sin datos de tokens» y el detalle «Todavía no hay mensajes con uso de este agente. No es un error.»
- **E2, pausa corta (AC36a):** una pausa que no supera el TTL no genera fila de reescritura.

### F8. Saber qué anota el registro de uso (US7; AC23, AC24)

Entrada: Opciones > «Registro de uso».
1. La tarjeta dice qué guarda (hora, tipo, id de sesión: avisos, clics en avisos, foco y pérdida de foco, y las acciones con intención: clics en tarjetas, píldoras o marcadores, cambio de pestaña, «Ir a Claude», respuestas), qué nunca guarda (textos, rutas, comandos), el tamaño máximo (1 MB), la ruta del archivo, cómo apagarlo (`usageLog: false` en `config.json`) y la hora de referencia de «nuevo».
2. «Ver las últimas marcas» muestra las últimas líneas reales: `14:31:05 · aviso · s-3f9a`.
- **E1, registro vacío:** «(todavía sin marcas)».
- **E2, registro apagado (AC24b):** la tarjeta cambia a «Apagado», no muestra marcas y dice que las notificaciones salen igual. Reproducible con el simulador.

### F9. Activar el % real del límite (US8, solo si pasa el Paso 0A; AC37, AC38, AC39, AC41, AC42, AC51)

Entrada: botón «Activar % real…» en la tarjeta del límite o en Opciones.
1. Se abre el diálogo **«Activar el % real del límite»** (S12). Muestra, antes de escribir: el JSON exacto de `statusLine`, el archivo (`%USERPROFILE%\.claude\settings.json`), **el backup propio de esta opción** (`settings.json.agent-office-statusline.bak`; el control tiene otro con otro nombre y uno nunca pisa al otro), cómo deshacer, y tres garantías (sin tokens, sin internet, y qué clave cambia). Foco inicial en **Cancelar**. Sobre la barra actual (AC39):
   - **Con `statusLine` propia:** «Ya tenés una statusLine y la voy a reemplazar por el encadenador de Agent Office. El encadenador ejecuta tu comando (`node ~/.claude/mi-barra.js`) y muestra su salida tal cual… Tu comando original queda guardado en `%APPDATA%\agent-office\config.json` y dentro del backup.» La garantía dice «Cambia solo la clave statusLine (la tuya se reemplaza por el encadenador)», ya no «solo agrega».
   - **Sin `statusLine`:** «No tenés una statusLine: agrego una que no muestra nada en la barra.» y la garantía «Solo agrega la clave statusLine».
2. «Escribir en settings.json y activar»: se hace el backup propio, se escribe la clave, el diálogo se cierra y el panel avisa: «Listo: % real activado. Backup propio en …. Tu comando original quedó guardado en … y en ese backup. Esperando el primer dato de Claude Code.».
3. La tarjeta del límite dice «Esperando el primer dato real…» y, cuando llega, «47 % (real) · se libera 16:00 · 7 días: 22 %». La alerta de F3 usa desde ahí el % real. Opciones pasa a «Activo», con archivo, backup y el botón «Deshacer % real…».
- **E1, cancelar:** no se escribe nada y se sigue con la estimación (AC37).
- **E2, `settings.json` con JSON roto (AC42):** diálogo «No escribí nada»: «`settings.json` tiene un error de formato (línea 14: falta una coma)…, tu archivo quedó exactamente igual y no se creó ningún backup. Corregilo y volvé a intentar.» Botones «Cerrar» y «Reintentar».
- **E3, archivo bloqueado por otro programa:** mismo diálogo, «Otro programa tiene abierto `settings.json`… Cerrá ese programa y volvé a intentar.»
- **E4, sin dato real (AC41):** si no llega dato (antes de la primera respuesta, API key, más de 10 min), la tarjeta vuelve a «~NN % (estimado)» y, si hubo un dato antes, agrega «Último dato real: 47 % hace 12 min. Sin datos nuevos, uso la estimación.» Sin error.
- **E5, Paso 0A falló:** el botón no existe. Opciones dice «No disponible: la prueba 0A no mostró el % real en la app de escritorio, así que seguís con la estimación.»

### F10. Deshacer el % real (US8; AC40)

Entrada: Opciones > «Deshacer % real…».
1. Diálogo de confirmación con **cuatro variantes** (regla de deshacer de Constraints y AC40, AC52):
   - **Exacto** (el archivo no cambió y el control está apagado): «Voy a restaurar `settings.json` exacto desde el backup de esta opción (`…statusline.bak`)».
   - **La otra opción sigue activa:** «El control sigue activo. Voy a quitar solo lo mío (tu `statusLine` original vuelve a su lugar) y no restauro el archivo entero: eso borraría lo del control. Cada opción tiene su propio backup.»
   - **Archivo editado a mano:** «Cambiaste settings.json después de activar esta opción… Tus cambios nuevos se conservan y no restauro el archivo entero.»
   - **El usuario reemplazó la `statusLine` del encadenador (AC40c):** «Tu status line cambió después de activar. No la toco: la dejo como está. Quito solo lo demás que agregó Agent Office…».
2. «Deshacer»: la tarjeta vuelve a «Apagado», el panel muestra el % estimado en 2 s o menos, y una nota deja escrito qué se hizo según la variante: «Restauré settings.json exacto desde el backup de esta opción.», «Quité solo lo mío y dejé el resto como estaba: el control sigue activo con su propio backup. No restauré el archivo entero.», «Quité solo lo que había agregado Agent Office y conservé tus cambios…» o «Tu status line cambió después de activar; la dejé como está…».
- **E1, cancelar:** no cambia nada.
- **E2, archivo editado:** variante del paso 1 (AC40b).
- **E3, statusLine reemplazada con el % real activo (AC40c):** desde que el usuario reemplazó el encadenador la tarjeta del límite muestra «~NN % (estimado)» más «El % real dejó de llegar: reemplazaste la status line y ya no pasa por Agent Office. Uso la estimación.»; Opciones pasa a «Dejó de llegar» con el mismo aviso y el botón «Deshacer % real…» sigue disponible.
- **E4, dos opciones activas:** las secuencias de AC52 (activar una, activar la otra, deshacer en cualquier orden) usan la variante «La otra opción sigue activa» en el primer deshacer y la «Exacto» en el último.

### F11. Activar el control desde Agent Office (US13, por partes según Paso 0B; AC43, AC49, AC51)

Entrada: Opciones > «Activar control…» (o el botón «Ir a Opciones» en una tarjeta de solo lectura).
1. Opciones lista qué parte está disponible: «Responder preguntas: disponible», «Aprobar o rechazar permisos: no disponible (falló la prueba 0B): ahí ves la pregunta y vas a Claude».
2. Diálogo **«Activar el control desde Agent Office»** (S13): los hooks exactos que escribe (solo los de las partes disponibles, tipo `command`, nunca `prompt` ni `agent`), el archivo, **el backup propio del control** (`settings.json.agent-office-control.bak`, distinto del del % real), el **canal local** (solo esta PC, no sale a internet), cómo funciona (el hook espera tu decisión hasta **120 s**, configurable entre 15 y 600 s, y la sesión queda pausada; pasado ese tiempo se libera y Claude pregunta él; «Contestar en Claude» lo libera antes; con Agent Office cerrado o el canal caído se libera en 2 s o menos; cada pregunta o permiso lanza un proceso corto; «Otra» se contesta en Claude), cómo deshacer y tres garantías (tus hooks quedan intactos). Foco inicial en **Cancelar**.
3. «Escribir en settings.json y activar»: backup propio, hooks, nota «Listo: control activado. Backup propio en …. Desde la próxima pregunta o permiso podés contestar acá.». Una pregunta que ya estaba esperando **sigue en solo lectura** («llegó antes de activar el control», AC59a).
- **E1, cancelar:** no se escribe nada.
- **E2, JSON roto o archivo bloqueado:** igual que F9 E2 y E3.
- **E3, una parte no pasó el 0B:** el diálogo dice qué queda afuera («Aprobar permisos no pasó la prueba 0B en tu equipo: esa parte sigue en solo lectura con Ir a Claude»).
- **E4, ninguna parte pasó el 0B:** Opciones dice «No disponible» y todo sigue en solo lectura (F15).

### F12. Responder una pregunta de Main desde la app (US13; AC44, AC46, AC47)

Entrada: tarjeta «Pregunta» con el control activo y el canal sano.
1. La tarjeta muestra **todas las preguntas de la llamada** (`AskUserQuestion` puede traer varias), cada una como un grupo: opciones de **elección única** (radio) o, si es `multiSelect`, **casillas** con la leyenda «(elegí una o más)». Se recorren con las flechas (radio) o con Tab y espacio (casillas). Hay **un solo** «Enviar respuesta», deshabilitado hasta que cada pregunta tenga al menos una opción elegida; la ayuda dice «Elegí al menos una opción en cada pregunta para poder enviar.» (AC60). Debajo: «Si ninguna sirve, «Otra» se contesta en Claude» (A8) y una cuenta regresiva «tenés 1:45 para contestar acá y después pasa a Claude» (el timeout de 120 s).
2. El usuario elige y toca «Enviar respuesta». Elegir no manda nada. El botón dice «Enviando…» (≈2 s como máximo). La tarjeta sale de «Te esperan», el marcador se va y aparece la nota: «Contestaste «Con IVA» y «Pro, Equipo» en Landing de precios. Main sigue con esas respuestas.» (AC44b).
3. En Claude no queda ningún diálogo que pida acción.
- **E1, «Contestar en Claude» (AC47b):** libera el hook sin decidir, la app muestra «Liberé la sesión sin decidir: Claude te muestra su propio diálogo.» y trae Claude al frente (ver F15 E1 si Windows no deja). Si el usuario vuelve a Agent Office sin contestar, la tarjeta queda en solo lectura con «Solo lectura: la mandaste a Claude sin decidir. Contestala allá.» (ya no «llegó antes de activar el control»).
- **E2, contestada en Claude (AC47a):** la tarjeta desaparece sola con la nota «Agent Office utilidad se contestó en Claude. Ya no te espera.».
- **E3, el canal con Claude no responde (AC55b):** la tarjeta pasa a solo lectura con «El canal con Claude no responde. Contestá en Claude: tu sesión no queda trabada.» y el botón «Reintentar canal». Opciones muestra «Sin respuesta», el mismo aviso y el botón. Al reintentar y volver a funcionar, Opciones vuelve a «Activo» y la pregunta siguiente se puede contestar acá (AC55c).
- **E3b, se venció el tiempo (AC55a):** a los 120 s sin contestar, el hook se libera, Claude muestra su diálogo y la tarjeta pasa a solo lectura con «Se venció el tiempo: contestá en Claude.» más la aclaración de que el hook se liberó y de dónde se cambia el valor. En esa tarjeta no hay botón de Enviar.
- **E3c, formato que no se puede contestar (AC44, A8):** si el Paso 0B mostró que el hook no contesta varias preguntas o `multiSelect`, esa tarjeta queda en solo lectura con «Esta pregunta tiene un formato que no puedo contestar desde acá: contestala en Claude.», mostrando todas las preguntas con sus opciones y la etiqueta «selección múltiple». Las de una sola pregunta de elección única siguen funcionando.
- **E4, Agent Office cerrado mientras la pregunta esperaba (AC46, AC59b):** Claude muestra su propio diálogo, sin demora extra. Al reabrir Agent Office, si sigue sin contestar, la pregunta aparece en solo lectura con «llegó con Agent Office cerrado y Claude la preguntó solo. Contestala en Claude.». La pregunta siguiente se contesta acá (AC59c).

### F13. Aprobar o rechazar un permiso (US13; AC45)

Entrada: tarjeta «Permiso» con el control activo.
1. La tarjeta dice «Pide permiso para usar **Bash**» y muestra el **comando completo** en un recuadro con scroll (enfocable con el teclado), con la aclaración «Ves el comando completo (N caracteres). Recorrelo hasta el final antes de decidir.» (AC45: la cola de un comando largo puede cambiar lo que se aprueba). El recorte a 60 caracteres queda solo en la línea de tiempo.
2. «Aprobar» o «Rechazar» (rojo, con texto) actúan al primer toque, sin confirmación extra (A14). Se aplica en 2 s o menos. Debajo, la cuenta regresiva del timeout («Tenés 1:45 para decidir acá y después pasa a Claude»). Nota: «Aprobaste el comando en … Se ejecuta ahora.» o «Rechazaste el comando en … Main lo ve como rechazado.».
- **E1, «Decidir en Claude»:** igual que F12 E1.
- **E2, canal caído o Agent Office cerrado:** igual que F12 E3 y E4, con «decidir» en lugar de «contestar».
- No hay aprobación en lote ni reglas automáticas (fuera de alcance 13): cada tarjeta se decide de a una.

### F14. Deshacer el control (US13; AC48)

Entrada: Opciones > «Deshacer control…».
1. Diálogo con las mismas variantes que F10 (exacto, «el % real sigue activo: quito solo mis hooks», archivo editado a mano), con el texto «Dejan de lanzarse procesos de hook. Lo que esté pendiente se contesta en Claude.». Los hooks propios del usuario quedan intactos y «Pregunta» vuelve a apagarse si se había prendido sola.
2. «Deshacer»: la tarjeta vuelve a «Apagado», las tarjetas pendientes pasan a solo lectura («Agent Office no está habilitado para contestar») y una nota confirma qué se hizo.
- **E1, cancelar:** no cambia nada.
- **E2, archivo editado:** aviso de que no se restaura el archivo entero.

### F15. Ver una pregunta o permiso en solo lectura e ir a Claude (US15; AC50)

Entrada: tarjeta «Pregunta» o «Permiso» sin control activo, o con una parte que falló el 0B, o con el canal caído.
1. La tarjeta muestra, leídas del transcript, **todas** las preguntas con sus opciones numeradas (lista, sin controles; la de selección múltiple lleva la etiqueta «selección múltiple») o la herramienta y el **comando completo** con scroll. Lleva la etiqueta «Solo lectura» y **una sola frase de motivo**: no activado, no disponible en esta instalación, llegó antes de activar, llegó con la app cerrada, la mandaste a Claude, se venció el tiempo, formato que no se puede contestar, o canal caído (AC50).
2. «Ir a Claude» trae al frente la ventana de la app de escritorio. El usuario contesta allá; la tarjeta desaparece sola (F1 paso 5). Si el motivo es «no activado», aparece también «Ir a Opciones».
- **E1, Windows no deja traer Claude al frente:** aparece en el panel el aviso «Abrí Claude: Agent Office utilidad te espera.» con «No es un error: buscala en la barra de tareas.» y «Entendido» (AC50).
- **E2, la sesión se contestó mientras tanto:** la tarjeta desaparece y no queda nada colgado.

## Screens

Todas viven en la ventana existente. Se agrega el panel a la derecha (390 px, colapsable) y dos elementos dentro de la escena.

| ID | Pantalla o vista | Propósito | Contenido principal | Acción principal | Historias |
|---|---|---|---|---|---|
| S1 | Ventana principal | Contenedor: barra lateral + 3D + panel | Barra de título de Windows, los tres bloques, pie permanente del panel | Mirar, elegir sesión | US1 a US10, US13, US15 |
| S2 | Barra lateral de edificios (existente, ampliada) | Elegir la sesión y ver su estado | Edificios con punto + **texto de estado** («te espera 1:30 · Pregunta»), pisos 5..1 con cuenta de «working» | Elegir edificio o piso | US1 |
| S3 | Escena 3D (existente, ampliada) | Ver la espera dentro de la oficina | Marcador sobre la oficina del jefe («? Pregunta · 1:30», «! Permiso», «✓ Terminó»), píldoras de otras sesiones que esperan, botones rotar/Fit existentes | Tocar el marcador o una píldora | US1 |
| S4 | Panel > Ahora > Te esperan | Responder «¿quién me espera?» de un vistazo | Lista ordenada por tiempo de espera, contador, notas de confirmación | Tocar una tarjeta | US1, US2, US15 |
| S5 | Tarjeta de espera (variantes: Pregunta con una o varias preguntas y `multiSelect`, Permiso, Terminó; interactiva o solo lectura) | Decidir o saltar a Claude | Preguntas con opciones (radio o casillas) o herramienta con el comando completo y scroll; cuenta regresiva del timeout; estado «Solo lectura» y su motivo | Enviar respuesta / Aprobar / Ir a Claude | US13, US15 |
| S6 | Panel > Ahora > Límite de 5 h | No chocar por sorpresa | Tokens de la ventana, barra con umbral, % (estimado o real), hora de reset, alerta, «¿Cómo se calcula?» | Ver quién consume / Activar % real | US3, US4, US8 |
| S7 | Panel > Consumo | Saber qué me consume | Rankings de 5 sesiones y 5 agentes, resumen de causas de la ventana | Abrir el desglose de la sesión | US5, US10 |
| S8 | Panel > Sesión > Qué pasó | Retomar sin releer | Línea de tiempo con marca «nuevo» y aviso de historia parcial | Leer | US6 |
| S9 | Panel > Sesión > Flujo | Ver cómo se pasan el trabajo | Diagrama en estrella, total de mensajes, tabla de mensajes | Leer | US9 |
| S10 | Panel > Sesión > Tokens | Ver causas por agente | Resumen de la sesión, tarjetas plegables por agente con desglose y total | Abrir/cerrar un agente | US10 |
| S11 | Panel > Opciones | Prender y deshacer las dos opciones opt-in; ver qué avisa y qué registra | Tarjetas: % real (con su backup), control (con su backup y el timeout), avisos de Windows (solo lectura, incluye «Pregunta» prendida sola), registro de uso (solo lectura, con la hora de referencia de «nuevo») | Activar / Deshacer | US7, US8, US13 |
| S12 | Diálogo «Activar el % real del límite» | Aviso previo, backup y confirmación | JSON a escribir, archivo, backup, encadenado, deshacer, garantías | Escribir y activar (o Cancelar) | US8 |
| S13 | Diálogo «Activar el control desde Agent Office» | Aviso previo, canal local y confirmación | Hooks, canal local, funcionamiento, backup, deshacer, garantías | Escribir y activar (o Cancelar) | US13 |
| S14 | Diálogo «Deshacer» (% real o control) | Deshacer sin pisar cambios | Variante con archivo sin cambios y variante con archivo editado | Deshacer (o Cancelar) | US8, US13 |
| S15 | Diálogo «No escribí nada» | Explicar por qué no se pudo escribir | Causa (JSON roto o archivo bloqueado), garantía de que no cambió nada | Reintentar / Cerrar | US8, US13 |
| S16 | Panel colapsado (píldora en la escena) | Devolverle espacio al 3D sin perder el aviso | «Panel · 2 te esperan · ~50 % (est.)» | Reabrir el panel | US1, US3 |
| S17 | Notificación de Windows | Avisar con la ventana minimizada | Título de la sesión y motivo, sin contenido; o «N sesiones te esperan»; o alerta del límite | Tocar para volver a la sesión | US2, US4 |

Contextos de Windows que el producto no diseña pero el prototipo simula para poder recorrer los flujos: el escritorio con la app minimizada o cerrada, y la ventana de Claude con su propio diálogo.

El **pie permanente** del panel (parte de S1) muestra siempre el modo: «Solo lectura. Sin red, sin tokens, sin escribir en `~/.claude`.» o, con una opción prendida, «Opciones activas: % real + control. Escribe en settings.json (con backup y deshacer)» sobre fondo ámbar, siempre con enlace a Opciones.

## States

Los cuatro estados (vacío, cargando, error, éxito) se definen donde aplican. «Cargando» casi no existe porque todo sale de archivos locales; solo se muestra cuando hay una espera real.

| Pantalla | Vacío | Cargando | Error o borde | Éxito |
|---|---|---|---|---|
| S2 | Sin sesiones: el texto existente de «sin edificios» | n/a | Sesión «cerrada» (punto rojo + texto) hasta que se retira la fila | Estado en texto + tiempo |
| S3 | Sin espera: sin marcador ni píldoras | n/a | n/a | Marcador con ícono y texto |
| S4 | «Nadie te espera» | n/a | Sesión cerrada: sale de la lista | Lista con tiempos corriendo |
| S5 interactiva | n/a | Botón «Enviando…» y controles deshabilitados | Canal caído: pasa a solo lectura con aviso y «Reintentar canal» | Nota de confirmación y la tarjeta sale |
| S5 solo lectura | n/a | n/a | «Ir a Claude» sin permiso de Windows: «Abrí Claude: … te espera»; motivos: se venció el tiempo, formato no contestable, la mandaste a Claude, canal caído | La tarjeta sale cuando se contesta en Claude |
| S6 | Sin choques: tokens y «Sin choques registrados para estimar el %…» | «Calculando el % (leyendo el historial)…» durante el escaneo; «Esperando el primer dato real…» (con % real activo) | Dato real viejo (más de 10 min): vuelve a «estimado» con la antigüedad; statusLine reemplazada: «El % real dejó de llegar» | Estimado; real; «Límite alcanzado · se libera 16:00»; alerta |
| S7 | Sin tokens en la ventana: «Todavía no hay consumo en esta ventana» | n/a | Agente sin datos: excluido del ranking | Rankings con tokens y % |
| S8 | «Todavía no pasó nada en esta sesión» | n/a | «Historia parcial: desde HH:MM» | Lista con «nuevo» |
| S9 | «Main no lanzó subagentes» | n/a | Informe «pendiente» | Diagrama + tabla |
| S10 | Sesión sin subagentes: solo Main | n/a | «sin datos de tokens» por agente | Desglose cuyas partes suman el total |
| S11 | n/a | n/a | «No disponible» por Paso 0A o 0B; canal sin respuesta; nota neutra si Windows no entregó un aviso | Estado Activo / Apagado, con nota de lo hecho |
| S12, S13 | n/a | n/a | Pasa a S15 si no se puede escribir | Cierra, nota de éxito |
| S14 | n/a | n/a | Variante «archivo editado» con aviso explícito | Cierra, nota de lo restaurado |
| S15 | n/a | n/a | Es el estado de error | «Reintentar» vuelve al diálogo de aviso |
| S16 | «nadie te espera · …» | n/a | n/a | Cuenta y % |
| S17 | n/a | n/a | Bloqueada por Windows: no se ve nada y la app no marca error (nota neutra en Opciones) | Título + motivo |

## Microcopy

Tono: español rioplatense, voseo, directo, sin mayúsculas innecesarias. Cada error dice qué pasó y qué hacer. Los rótulos ya existentes de la app (Buildings, Fit, etc.) quedan en inglés, tal como están hoy.

| Dónde | Texto |
|---|---|
| Sección | «Te esperan», «Límite de 5 h», «Quién consume en esta ventana», «Dónde se van los tokens» |
| Pestañas del panel | Ahora, Consumo, Sesión, Opciones. Subpestañas: Qué pasó, Flujo, Tokens |
| Motivos | Pregunta, Permiso, Terminó |
| Vacío (Te esperan) | «Nadie te espera» / «Cuando una sesión necesite algo tuyo, aparece acá y como marcador en el 3D.» |
| Sesión espera (barra lateral) | «te espera 1:30 · Pregunta» |
| Marcador 3D | «? Pregunta · 1:30», «! Permiso · 0:45», «✓ Terminó · 0:30» |
| Tarjeta Terminó | «Main terminó su turno y no hay subagentes trabajando. Cuando le escribas, la sesión sale de esta lista.» Botones: «Ver qué pasó», «Ir a Claude» |
| Pregunta interactiva | Pregunta como leyenda del grupo; «Enviar respuesta», «Contestar en Claude»; ayuda: «Elegí una opción para poder enviar. Mientras no contestes, la sesión queda pausada. «Contestar en Claude» la libera y decidís allá.» |
| Permiso interactivo | «Pide permiso para usar Bash:» + comando completo con scroll; «Aprobar», «Rechazar», «Decidir en Claude»; «Ves el comando completo (N caracteres). Recorrelo hasta el final antes de decidir.»; «Tenés 1:45 para decidir acá y después pasa a Claude.» |
| Solo lectura | Etiqueta «Solo lectura» + motivo (ver abajo) + «Ir a Claude» |
| Motivo: control apagado | «Solo lectura: Agent Office no está habilitado para contestar. Podés activar el control en Opciones.» |
| Motivo: parte no disponible | «Solo lectura: contestar desde acá no está disponible en esta instalación (no pasó la prueba 0B). Contestá en Claude.» |
| Motivo: llegó antes | «Solo lectura: llegó antes de activar el control. Contestala en Claude; las próximas las contestás acá.» |
| Motivo: app cerrada | «Solo lectura: llegó con Agent Office cerrado y Claude la preguntó solo. Contestala en Claude.» |
| Motivo: se venció el tiempo (AC55a) | «Se venció el tiempo: contestá en Claude. El hook se liberó y Claude te muestra su propio diálogo (se cambia en config.json, entre 15 y 600 s).» |
| Motivo: formato (AC44) | «Esta pregunta tiene un formato que no puedo contestar desde acá: contestala en Claude.» |
| Motivo: la mandaste a Claude | «Solo lectura: la mandaste a Claude sin decidir. Contestala allá.» |
| Varias preguntas | Leyenda por pregunta; multiSelect: «(elegí una o más)» / en solo lectura, etiqueta «selección múltiple». Ayuda: «Elegí al menos una opción en cada pregunta para poder enviar. Si ninguna sirve, «Otra» se contesta en Claude. Mientras no contestes, la sesión queda pausada: tenés 1:45 para contestar acá y después pasa a Claude.» Nota: «Contestaste «Con IVA» y «Pro, Equipo» en {sesión}. Main sigue con esas respuestas.» |
| Terminó vence | «Sale de esta lista cuando le escribas o a las 4 h (se cambia en config.json); después queda «inactiva».» |
| Calculando | «Calculando el % (leyendo el historial)…» / «Tarda hasta 30 s. «Te esperan» y la escena no esperan al cálculo.» |
| Nuevo | ««Nuevo» = desde la última vez que miraste la app (14:05).» / sin registro: ««Nuevo» marca todo lo que hay: el registro de uso está apagado y no sé cuándo miraste por última vez.» |
| Backups | «Backup de esta opción: …settings.json.agent-office-statusline.bak» / «…agent-office-control.bak». «El control tiene su propio backup, con otro nombre: uno nunca pisa al otro.» |
| % real con barra propia | «Ya tenés una statusLine y la voy a reemplazar por el encadenador de Agent Office. El encadenador ejecuta tu comando (…) y muestra su salida tal cual, así tu barra sigue igual. Tu comando original queda guardado en …\agent-office\config.json y dentro del backup.» |
| Deshacer, variantes | «Quité solo lo mío y dejé el resto como estaba: el control sigue activo con su propio backup. No restauré el archivo entero.» · «Tu status line cambió después de activar; la dejé como está. Quité solo lo demás que había agregado Agent Office.» · «El % real dejó de llegar: reemplazaste la status line y ya no pasa por el encadenador de Agent Office.» |
| Pregunta prendida sola | «Pregunta: prendido solo, mientras el control esté activo (Claude no avisa mientras Agent Office retiene la pregunta; al deshacer el control vuelve a apagado)» |
| Error: canal caído | «El canal con Claude no responde. Contestá en Claude: tu sesión no queda trabada.» + «Reintentar canal» |
| Ir a Claude falla | «Abrí Claude: {sesión} te espera.» / «Windows no me dejó traer la ventana al frente. No es un error: buscala en la barra de tareas.» |
| Confirmaciones | «Contestaste «{opción}» en {sesión}. Main sigue con esa respuesta.» · «Aprobaste el comando en {sesión}. Se ejecuta ahora.» · «Rechazaste el comando en {sesión}. Main lo ve como rechazado.» · «{sesión} ya se contestó en Claude. Ya no te espera.» · «Liberé la sesión sin decidir: Claude te muestra su propio diálogo.» |
| Límite, estimado | «600 k tokens en esta ventana» · «~50 % (estimado) · se libera ~16:02 (estimado)» |
| Límite, sin choques | «Sin choques registrados para estimar el %. Cuando choques con el límite una vez, empiezo a estimarlo.» |
| Límite, choque | «Límite alcanzado · se libera 16:00» |
| Límite, real | «47 % (real) · se libera 16:00» · «7 días: 22 %» |
| Límite, real viejo | «Último dato real: 47 % hace 12 min. Sin datos nuevos, uso la estimación.» |
| Límite, esperando dato | «Esperando el primer dato real: llega con la próxima respuesta de Claude Code.» |
| Alerta | «Alerta: pasaste el 80 %. Pausá, compactá o cerrá sesiones antes de chocar.» |
| Cálculo | «El techo es la mediana de lo que consumiste en las 5 h previas a cada choque registrado en disco (3 choques). El uso desde claude.ai u otros dispositivos no aparece acá, así que puede fallar. Todo se calcula en tu PC, sin red y sin tokens.» |
| Línea de tiempo | «nuevo» · «Historia parcial: desde 10:31. El transcript es más grande que lo que leo de entrada; lo anterior no se muestra.» · «Todavía no pasó nada en esta sesión» · «Se arma en vivo desde los transcripts. Nada de esto se guarda en disco.» |
| Flujo | «Mensajes entre agentes: ~9 k tokens, 2,5 % de los tokens de la sesión.» · «subagente a subagente» · «informe pendiente» · «Main no lanzó subagentes» |
| Tokens | «Arranque», «Crecimiento del contexto», «Reescritura de caché tras pausa», «Informe devuelto a Main», «Total» · «sin datos de tokens» |
| Opciones, intro | «Por defecto Agent Office no escribe nada en tu configuración de Claude. Estas dos opciones son independientes, apagadas por defecto, y cada una te muestra qué hace antes de activarse.» |
| Diálogo % real | Título «Activar el % real del límite»; secciones «Qué voy a escribir», «En qué archivo», «Backup», «Tu barra actual», «Cómo deshacer»; garantías «No gasta tokens.», «No usa internet: solo recibe datos de Claude Code en esta PC.», «Cambia solo la clave statusLine (la tuya se reemplaza por el encadenador); no toca las demás.» (sin barra propia: «Solo agrega la clave statusLine…»); botones «Cancelar» y «Escribir en settings.json y activar» |
| Diálogo control | Título «Activar el control desde Agent Office»; secciones «Qué voy a escribir», «Qué queda afuera», «Canal local», «Cómo funciona», «Cómo deshacer»; garantía «Decidís vos cada pregunta y cada permiso, de a uno.» |
| Diálogo deshacer | Sin cambios: «Voy a restaurar settings.json exacto desde el backup de esta opción…». La otra opción activa: «El control sigue activo. Voy a quitar solo lo mío… y no restauro el archivo entero.» Editado: «Cambiaste settings.json después de activar esta opción. Voy a quitar solo lo que agregó Agent Office… Tus cambios nuevos se conservan y no restauro el archivo entero.» Botones «Cancelar» y «Deshacer» |
| Error de escritura | Título «No escribí nada». JSON: «settings.json tiene un error de formato (línea 14: falta una coma). Por eso no lo toqué.» + «Corregilo (o restauralo desde tu último backup) y volvé a intentar.». Bloqueado: «Otro programa tiene abierto settings.json y no me deja escribir.» + «Cerrá ese programa y volvé a intentar.». Siempre: «Tu archivo quedó exactamente igual que antes y no se creó ningún backup.» Botones «Cerrar» y «Reintentar» |
| Avisos de Windows | «Los avisos solo muestran el título de la sesión y el motivo, nunca preguntas, rutas ni comandos. Windows puede guardarlos en su centro de notificaciones. Avisan siempre que la ventana no tenga el foco (minimizada, visible al costado o tapada); con el foco alcanza con el panel. Si la cerrás, Agent Office se cierra y no avisa. Esta tarjeta solo informa.» · nota: «Windows no entregó el último aviso (revisá Configuración > Sistema > Notificaciones). Igual lo ves en «Te esperan».» |
| Registro de uso | «Guarda solo hora, tipo y id de sesión: cuándo te avisó, cuándo tocaste un aviso, cuándo perdió o recuperó el foco la ventana y tus acciones… Nunca textos, rutas ni comandos. Pesa 1 MB como máximo.» · «Referencia de «nuevo»: la última vez que miraste la app, a las 14:05:00.» · apagado: «No se anota nada. Las notificaciones salen igual. Como no sé cuándo miraste por última vez, «nuevo» marca todo lo que hay.» |
| Notificaciones de Windows | Una sesión: título = título de la sesión; cuerpo = «Te espera · Permiso». Varias: «Agent Office» / «3 sesiones te esperan». Límite: «Llevás ~80 % del límite de 5 h (estimado) · se libera ~16:02» |
| Pie permanente | «Solo lectura. Sin red, sin tokens, sin escribir en ~/.claude.» / «Opciones activas: % real + control. Escribe en settings.json (con backup y deshacer).» |
| Panel colapsado | «Panel · 2 te esperan · ~50 % (est.)» |
| Tooltips de la ventana | Cerrar: «Cerrar: Agent Office se cierra y deja de avisar». Minimizar: «Minimizar: Agent Office sigue vigilando y te avisa con notificaciones de Windows» |

## Accessibility

- **Nada solo con color.** Los motivos llevan ícono con forma distinta (?, !, ✓) y texto. El marcador 3D tiene texto. La barra lateral agrega texto al punto de color. El umbral del límite es una marca con la leyenda «umbral 80 %»; la zona en alerta usa barra rayada y el texto «Alerta». El flujo subagente a subagente tiene etiqueta de texto y la tabla repite el diagrama.
- **Teclado.** Orden de tabulación: barra de título > barra lateral (edificios y pisos) > escena (píldoras, marcador, rotar/Fit) > panel (Ocultar, pestañas, contenido, pie). Las pestañas y subpestañas son `role="tab"` con `aria-selected`. Las opciones de una pregunta de elección única son un grupo de radios que se mueven con las flechas; las de una `multiSelect` son casillas (Tab y espacio) dentro de un `fieldset` con la leyenda de la pregunta. El comando completo de un permiso es un recuadro con scroll al que se llega con Tab y se recorre con las flechas. La cuenta regresiva del timeout no se anuncia cada segundo. `Esc` cierra un diálogo. El foco se conserva al redibujar y vuelve al botón de origen al cerrar un diálogo.
- **Diálogos.** `role="dialog"`, `aria-modal`, el foco inicial va a **Cancelar** (nunca a la acción que escribe) y el diálogo contiene el foco. Los errores usan `role="alert"`.
- **Lectores de pantalla.** «Te esperan» es una región con `aria-live="polite"`: se anuncian altas, bajas y confirmaciones. El tiempo que corre no se anuncia cada segundo (solo se anuncian los cambios de la lista). El diagrama de flujo tiene `role="img"` con descripción y la tabla es la versión leíble.
- **Contraste.** Tema oscuro existente: texto principal `#e6e9ef` sobre `#232833` (más de 10:1); texto atenuado `#8d97a8` sobre `#232833` (más de 4,5:1). Los bordes ámbar y naranja de las alertas son decorativos; el significado está en el texto. Verificar con la herramienta de contraste del builder antes de cerrar.
- **Tamaños táctiles y de clic.** Botones y filas con 32 px de alto como mínimo en escritorio y 40 px en la hoja inferior de ancho teléfono. Las tarjetas de espera son un solo botón de ancho completo.
- **Movimiento.** El marcador del 3D rebota suavemente; con `prefers-reduced-motion` queda quieto.
- **Notificaciones.** Son texto plano: título de la sesión y motivo; se leen sin abrir la app.
- **Panel colapsable.** «Ocultar» devuelve el espacio al 3D y deja una píldora con el resumen (S16).

## Assumptions

Decisiones de diseño que hay que confirmar:

1. **Estructura del panel.** Un panel derecho de 390 px con cuatro pestañas (Ahora, Consumo, Sesión, Opciones). «Ahora» junta lo que se mira de reojo («Te esperan» y el límite); lo demás se abre a pedido. Alternativa descartada: una columna larga con todo.
2. **Ya resueltas por el spec (Gate 3), sin cambios en el diseño:** responder en dos pasos y aprobar de uno (A14, AC60), píldoras en la escena y píldora del panel colapsado (AC56, AC57), «Ir a Claude» en «Terminó» (AC58), nota neutra cuando Windows bloquea avisos y tarjetas de solo lectura en Opciones (AC61), pregunta en solo lectura si llegó antes de activar o con la app cerrada (AC59), texto nuevo en castellano y rótulos existentes en inglés (AC62).
3. **Estados de la ventana.** El prototipo distingue cuatro: con foco, visible sin foco (la barra de título se atenúa y dice «sin foco»), tapada y minimizada. Dar clic en la ventana le da el foco. «Tapada» y «sin foco» no se pueden detectar igual en Windows; la implementación puede usar la visibilidad de la ventana y el foco del sistema, y el diseño solo exige que se notifique en los tres casos sin foco (AC9).
4. **Pasos 0 simulados.** El simulador representa tres resultados de prueba que el diseño no puede conocer: 0B (el hook no contesta varias preguntas o `multiSelect`; Claude no avisa mientras el hook retiene la pregunta) y 0C (el clic en el aviso no puede elegir la sesión). Cada uno tiene su camino en el diseño, descrito en F2, F11 y F12.
5. **Timeout visible.** La tarjeta interactiva muestra una cuenta regresiva del timeout del control (120 s por defecto). Es un agregado de la UX al spec (AC55 solo pide el estado final). Si no te gusta que haya un reloj más, se deja solo el motivo «Se venció el tiempo».
6. **«Nuevo» en la línea de tiempo.** Una línea de ayuda arriba de la lista dice desde cuándo cuenta «nuevo», y Opciones muestra esa hora. Los eventos se comparan con la hora de la última pérdida de foco; el prototipo la deriva de su propio reloj, la app real la lee del registro de uso.
7. **Ventana lateral (F26 de la revisión).** Con la ventana angosta (hasta 900 px) el panel pasa a una hoja inferior que tapa parte del 3D; con 600x900 «Te esperan» debería quedar visible sin scroll (no lo medí en un navegador). El spec todavía no tiene un criterio con ese tamaño (AC30 mide 1200x760 y 900x600); si se agrega, el diseño lo cumple con la hoja inferior.
8. **«Terminó» a las 4 h.** La tarjeta lo avisa; el reloj de espera pasa a h:mm:ss desde la hora. Al vencer, la fila pasa a «inactiva» sin aviso ni nota.
9. **Nombres provisorios.** El helper `agent-office-statusline.exe`, los backups `settings.json.agent-office-statusline.bak` y `settings.json.agent-office-control.bak`, la configuración propia `%APPDATA%\agent-office\config.json`, el registro `%LOCALAPPDATA%\agent-office\usage.log` y los ids de sesión del registro (`s-3f9a`) son ejemplos. Los nombres finales los define el architect; los avisos de la app y el README deben usar los mismos (AC51).
10. **Idioma.** Lo nuevo está en español rioplatense; los rótulos ya existentes (Buildings, Fit, F3) siguen en inglés. Traducirlos queda fuera de este diseño.
11. **«Terminó» apagado.** Con la ventana minimizada no hay aviso de Agent Office cuando una sesión termina (lo avisa Claude, A4). La sesión sí aparece en «Te esperan» al volver.
12. **Reloj de espera.** El tiempo se muestra m:ss y corre solo mientras el panel está visible. El prototipo lo actualiza cada segundo; en la app real debe respetar el render bajo demanda (AC30): sin cambios, sin frames.
13. **Ancho de teléfono.** Agent Office es una app de escritorio; el ancho de teléfono del prototipo equivale a una ventana lateral muy angosta. A ese ancho la barra lateral y el panel se vuelven superpuestos. No se prevé una app móvil.
14. **Horas del límite.** «Se libera ~HH:MM» está en hora local y se redondea al minuto; el real es una hora exacta («16:00»).
15. **Pausa del hook.** El texto del diálogo de control advierte que mientras el hook espera la sesión queda pausada (R7 y R9). Si el Paso 0B muestra otro comportamiento del diálogo de escritorio, ese texto cambia.

## Traceability

### Por historia

| Historia | Pantallas | Flujos |
|---|---|---|
| US1 | S2, S3, S4, S5, S16 | F1 (AC1 a AC6, AC54, AC56 a AC58) |
| US2 | S4, S11, S17 | F2 (AC7 a AC11, AC61) |
| US3 | S6, S16 | F3 |
| US4 | S6, S17 | F3 |
| US5 | S7 | F4 |
| US6 | S8 | F5 |
| US7 | S11 | F2 (marcas), F8 |
| US8 | S6, S11, S12, S14, S15 | F9, F10 |
| US9 | S9 | F6 |
| US10 | S7, S10 | F4, F7 |
| US13 | S5, S11, S13, S14, S15 | F11, F12, F13, F14 |
| US15 | S5 | F15 |

### Por criterio de aceptación

«Sin UI» significa que el criterio se verifica por comando, test o archivo y el diseño no agrega pantalla; el diseño solo debe no contradecirlo.

| AC | Pantallas y pasos |
|---|---|
| AC1 | S4, S5: F1 pasos 2 y 3 (orden y motivo, A no aparece) |
| AC2 | S4: F1 paso 5 |
| AC3 | S3 (marcador con texto, distinto por motivo), S2 (texto «te espera» y tiempo): F1 paso 1 |
| AC4 | S4: F1 paso 4 |
| AC5 | S4: F1 E3 (la ausencia de tarjeta es el comportamiento) |
| AC6 | S4: F1 E1 y E2 |
| AC7 | S17, S4: F2 pasos 1 y 3 |
| AC8 | S17: F2 paso 3 y E1b (fallback del Paso 0C: Ahora sin seleccionar sesión) |
| AC9 | F2 E1 (con foco, visible sin foco, tapada) y E2 (ráfaga) (S17) |
| AC10 | S11 (avisos, solo lectura), F2 E3 y E3b (Pregunta prendida sola con el control activo) |
| AC11 | S11, F2 E4 |
| AC12 | S6: F3 paso 1 |
| AC13 | S6: F3 E2 |
| AC14 | S6: F3 E1 |
| AC15 | S6, S17: F3 paso 2 |
| AC16 | S6 (marca «umbral 80 %»), F3 E3; el log de la advertencia es sin UI |
| AC17 | S7: F4 paso 1 |
| AC18 | S7: F4 (el desglose solo cuenta la ventana), sin pantalla propia |
| AC19 | S8: F5 paso 1 |
| AC20 | S8: F5 paso 2 («nuevo» desde la última vez que miraste la app; la hora de referencia está en S11), F5 E0 (registro apagado: todo «nuevo») |
| AC21 | S8: F5 E1 |
| AC22 | S8: F5 paso 3 (texto); el chequeo de archivos es sin UI |
| AC23 | S11: F8 pasos 1 y 2 (muestra marcas sin contenido, incluidas pérdida de foco y acciones con intención) |
| AC24 | S11: F8 E1 y E2 |
| AC25 | Sin UI (comando de `office-dump`) |
| AC26 | Sin UI (el diseño no agrega ninguna llamada a red; el prototipo no hace ninguna) |
| AC27 | Sin UI; el pie permanente de S1 declara el modo solo lectura y el diseño de S11 lo respeta |
| AC28 | Sin UI |
| AC29 | Sin UI; nota 12 de Assumptions (render bajo demanda, sin librerías de UI) |
| AC30 | S16 y la columna del panel colapsado (el panel se puede ocultar a 900 px); el chequeo es de build |
| AC31 | Sin UI; los textos de S11 («Avisos de Windows», «Registro de uso») coinciden con lo que debe decir el README |
| AC32 | S9: F6 pasos 1 a 4 |
| AC33 | S9: F6 E1 y E2 (el anidamiento está en el paso 2) |
| AC34 | S10: F7 paso 2 |
| AC35 | S10 (sesión) y S7 (ventana): F7 paso 1, F4 paso 2 |
| AC36 | S10: F7 E1 y E2 |
| AC37 | S12: F9 pasos 1 y 2, E1 (backup propio de la opción) |
| AC38 | S6: F9 paso 3 |
| AC39 | S12 («Tu barra actual»: la `statusLine` se reemplaza por el encadenador y dice dónde queda el original): F9 paso 1; la salida del comando es sin UI |
| AC40 | S14: F10 pasos 1 y 2 (variantes exacto, otra opción activa, editado, statusLine cambiada), E2 y E3; el panel vuelve al estimado |
| AC41 | S6: F9 E4 |
| AC42 | S15: F9 E2 y E3 |
| AC43 | S13: F11 pasos 2 y 3 (backup propio del control, hooks propios intactos) |
| AC44 | S5: F12 pasos 1 a 3 (varias preguntas y `multiSelect`), E3c (formato no contestable) |
| AC45 | S5: F13 pasos 1 y 2 (comando completo con scroll, un toque) |
| AC46 | S5 (solo lectura «llegó con Agent Office cerrado»): F12 E4 |
| AC47 | S5: F12 E1 y E2 |
| AC48 | S14: F14 (variantes exacto y otra opción activa) |
| AC49 | S13 (texto del canal local); el chequeo de puertos y del mensaje inválido es sin UI |
| AC50 | S5 (solo lectura: una o varias preguntas, comando completo, un motivo) y aviso «Abrí Claude»: F15 pasos 1 y 2, E1 |
| AC51 | S12, S13, S11: los avisos de la app deben ser idénticos al README (assumption 9) |
| AC52 | S12, S13, S14, S11: F10 E4 (secuencias con las dos opciones; cada una con su backup y su variante de deshacer). El hash final es sin UI |
| AC53 | S6: F3 E0 («Calculando el % (leyendo el historial)…» mientras «Te esperan» y la escena ya funcionan); S16 («calculando el %…») |
| AC54 | S2, S4, S5: F1 paso 3 y E5 («Terminó» visible a los 45 min, vence a las 4 h y queda «inactiva») |
| AC55 | S5, S11: F12 E3 (canal caído, «Reintentar canal», Opciones vuelve a «Activo»), E3b (timeout vencido), F11 paso 2 (timeout 120 s, 15 a 600 s) |
| AC56 | S3: F1 E6 (píldoras de las otras sesiones) |
| AC57 | S16: F1 E4 |
| AC58 | S5: F1 paso 3 («Ver qué pasó» e «Ir a Claude» en «Terminó») |
| AC59 | S5: F11 paso 3 (llegó antes de activar), F12 E4 (llegó con la app cerrada; la siguiente se contesta acá) |
| AC60 | S5: F12 pasos 1 y 2 (grupo de opciones, «Enviar respuesta» deshabilitado hasta responder todo, «Enviando…»), F13 paso 2 (permiso de un toque) |
| AC61 | S11: F2 E3, E3b y E4; F8 (tarjetas de solo lectura de avisos y de registro, nota neutra de Windows) |
| AC62 | Todas las pantallas: texto nuevo en castellano rioplatense; los rótulos existentes (Buildings, Fit, F3) no cambian (Microcopy, assumption 10) |
| AC63 | Sin UI propia (lo ejecuta el desinstalador). El diseño no agrega nada: S11 queda sin tarjetas activas después de desinstalar y la app avisa en el README (assumption 9) |

## Revisions

- **Ciclo 1 (revisión `review.md` y spec con AC52 a AC63).** Prototipo y flujos actualizados en el lugar:
  - **Backups por opción (F1 de la revisión, AC52):** rutas distintas para % real y control en los diálogos, en Opciones y en las notas; cuatro variantes de deshacer (exacto, la otra opción sigue activa, archivo editado, statusLine cambiada) con la copia «quité solo lo mío». F10 y F14.
  - **statusLine reemplazada (F2, AC39 y AC40c):** el aviso dice que se reemplaza por el encadenador y dónde queda el original; aviso «Tu status line cambió después de activar; la dejé como está»; estado «Dejó de llegar» en Opciones y en el límite. F9 y F10.
  - **Varias preguntas y `multiSelect` (F5, AC44 y AC60):** casillas, un solo «Enviar respuesta», «Otra» en Claude, caso «formato que no puedo contestar» en solo lectura. F12 y F15.
  - **Comando completo con scroll (F14, AC45):** el recorte de 60 caracteres queda solo en la línea de tiempo. F13.
  - **Aviso siempre que no haya foco (F9, AC9):** el simulador agrega «visible sin foco» y «tapada»; la barra de título muestra «sin foco». F2.
  - **«Terminó» vence a las 4 h (F6, AC54):** F1.
  - **«Nuevo» desde la última vez (F20, A7, AC20):** línea de ayuda en la línea de tiempo y hora de referencia en Opciones; el registro apagado marca todo como nuevo. F5 y F8.
  - **Escaneo histórico (F4, AC53):** «Calculando el % (leyendo el historial)…». F3.
  - **Paso 0C (F15, AC8):** fallback de la notificación; la alerta de límite abre Ahora > Límite (F22). F2.
  - **«Pregunta» prendida sola (F7, AC10c)** en Opciones. F2.
  - **Timeout del control (F8, AC55):** 120 s con cuenta regresiva, motivo «Se venció el tiempo: contestá en Claude» y «Reintentar canal». F11, F12.
  - **F22 de la revisión:** tras «Contestar en Claude» la tarjeta dice «la mandaste a Claude sin decidir» (antes decía un motivo falso).
  - **Traza:** AC1 a AC63 completa. Las notas de Assumptions que el spec ya resolvió pasaron a una sola línea.

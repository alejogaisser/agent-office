#!/usr/bin/env node
// Writes synthetic Claude Code sessions into a temp "claude dir" so the app can be
// exercised without real sessions. No dependencies.
//
//   node scripts/simulate.mjs --root <dir> --scenario static|basic|spawn|multi|soak|multi-team|walk|relay|overflow
//        [--subs 4] [--sessions 5] [--idle] [--minutes 10] [--rate 20] [--seconds 600]
//
// Run the app with AGENT_OFFICE_CLAUDE_DIR=<dir>. Prints `WRITE <iso> <what>` per write.
import fs from "node:fs";
import path from "node:path";

const args = Object.fromEntries(
  process.argv.slice(2).reduce((acc, a, i, all) => {
    if (a.startsWith("--")) {
      const next = all[i + 1];
      acc.push([a.slice(2), next && !next.startsWith("--") ? next : true]);
    }
    return acc;
  }, []),
);
const root = args.root;
const scenario = args.scenario ?? "basic";
if (!root || typeof root !== "string") {
  console.error("usage: simulate.mjs --root <dir> --scenario static|basic|spawn|multi|soak|multi-team|walk|relay|overflow");
  process.exit(2);
}
const projectsDir = path.join(root, "projects");
const sessionsDir = path.join(root, "sessions");
fs.mkdirSync(projectsDir, { recursive: true });
fs.mkdirSync(sessionsDir, { recursive: true });

const registryFiles = [];
const iso = () => new Date().toISOString();
const log = (what) => console.log(`WRITE ${iso()} ${what}`);
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
let counter = 0;
const uid = (p) => `${p}_${Date.now().toString(36)}${(counter++).toString(36)}`;

function cleanup() {
  for (const f of registryFiles) {
    try {
      fs.unlinkSync(f);
    } catch {}
  }
}
process.on("exit", cleanup);
for (const s of ["SIGINT", "SIGTERM"]) process.on(s, () => process.exit(0));

// ---- line builders -------------------------------------------------------
const usage = (o = 5) => ({ input_tokens: 10, output_tokens: o, cache_read_input_tokens: 1000, cache_creation_input_tokens: 0 });
const toolUse = (id, name, input, stop = "tool_use") =>
  JSON.stringify({
    type: "assistant",
    timestamp: iso(),
    message: { id: uid("msg"), model: "sim-model", stop_reason: stop, usage: usage(), content: [{ type: "tool_use", id, name, input }] },
  });
const toolResult = (id, text = "ok") =>
  JSON.stringify({
    type: "user",
    timestamp: iso(),
    message: { role: "user", content: [{ type: "tool_result", tool_use_id: id, content: text }] },
  });
const userPrompt = (cwd, text = "simulated prompt") =>
  JSON.stringify({ type: "user", timestamp: iso(), cwd, gitBranch: "main", message: { role: "user", content: text } });
const endTurn = () =>
  JSON.stringify({
    type: "assistant",
    timestamp: iso(),
    message: { id: uid("msg"), model: "sim-model", stop_reason: "end_turn", usage: usage(), content: [{ type: "text", text: "done" }] },
  });
const stopHook = () => JSON.stringify({ type: "system", subtype: "stop_hook_summary", timestamp: iso() });

const ROLES = [
  ["gaisser-agents:software-architect", "Plan the change"],
  ["gaisser-agents:software-engineer", "Implement the plan"],
  ["gaisser-agents:software-test-engineer", "Verify the build"],
  ["gaisser-agents:software-debugger", "Find the bug"],
  ["Explore", "Look around"],
  ["gaisser-agents:software-docs-writer", "Write the docs"],
];

function append(file, lines, what) {
  fs.appendFileSync(file, lines.join("\n") + "\n");
  log(what);
}

function makeSession(index, name) {
  const cwd = `C:\\sim\\${name}`;
  const slug = cwd.replace(/[^A-Za-z0-9]/g, "-");
  const sid = `00000000-0000-4000-8000-${String(1000 + index).padStart(12, "0")}`;
  const dir = path.join(projectsDir, slug);
  fs.mkdirSync(path.join(dir, sid, "subagents"), { recursive: true });
  const main = path.join(dir, `${sid}.jsonl`);
  fs.writeFileSync(main, userPrompt(cwd) + "\n");
  const regFile = path.join(sessionsDir, `${process.pid}${index === 0 ? "" : String(index).padStart(3, "0")}.json`);
  fs.writeFileSync(
    regFile,
    JSON.stringify({ pid: process.pid, sessionId: sid, cwd, name: `Sim ${name}`, status: "busy", startedAt: Date.now() - index * 60000, updatedAt: Date.now() }),
  );
  registryFiles.push(regFile);
  return { sid, cwd, main, subDir: path.join(dir, sid, "subagents"), subs: [] };
}

/** Spawns a foreground (or background) subagent: parent tool_use + subagent files. */
function spawnSub(s, i, { idle = false, background = false, role } = {}) {
  const [type, desc] = role ?? ROLES[i % ROLES.length];
  const agentId = `a${String(i).padStart(3, "0")}${Math.random().toString(16).slice(2, 8)}`;
  const toolUseId = uid("toolu");
  append(
    s.main,
    [toolUse(toolUseId, "Agent", { subagent_type: type, description: desc, prompt: "simulated", run_in_background: background })],
    `parent spawn ${agentId}`,
  );
  fs.writeFileSync(
    path.join(s.subDir, `agent-${agentId}.meta.json`),
    JSON.stringify({ agentType: type, description: desc, toolUseId, spawnDepth: 1 }),
  );
  const file = path.join(s.subDir, `agent-${agentId}.jsonl`);
  const body = [userPrompt(s.cwd, "simulated sub prompt")];
  body.push(idle ? endTurn() : toolUse(uid("toolu"), "Read", { file_path: `src/file${i}.ts` }));
  append(file, body, `subagent ${agentId} (${type})`);
  s.subs.push({ agentId, file, toolUseId });
  return s.subs[s.subs.length - 1];
}

async function staticScenario() {
  const sessions = Number(args.sessions ?? 5);
  const subs = Number(args.subs ?? 4);
  const idle = args.idle === true;
  for (let n = 0; n < sessions; n++) {
    const s = makeSession(n, `static${n}`);
    for (let i = 0; i < subs; i++) spawnSub(s, i, { idle: idle || !(n === 0 && i === 0) });
    // main: working only in session 0 unless --idle
    if (idle || n > 0) append(s.main, [endTurn(), stopHook()], `main ${n} idle`);
    else append(s.main, [toolUse(uid("toolu"), "Bash", { command: "npm test" })], `main ${n} running`);
  }
  console.log("READY static");
  setInterval(() => {}, 1 << 30); // keep the process (and its registry files) alive
  await new Promise(() => {});
}

async function basicScenario(s, seconds = 30) {
  const end = Date.now() + seconds * 1000;
  while (Date.now() < end) {
    const id = uid("toolu");
    append(s.main, [toolUse(id, "Edit", { file_path: "src/app.ts", old_string: "a", new_string: "b" })], "main edit");
    await sleep(1500);
    append(s.main, [toolResult(id)], "main edit result");
    await sleep(1500);
  }
  append(s.main, [endTurn(), stopHook()], "main end of turn");
}

async function spawnScenario() {
  const s = makeSession(0, "spawn");
  console.log("READY spawn");
  await sleep(3000);
  for (let i = 0; i < 4; i++) {
    const sub = spawnSub(s, i);
    await sleep(3000);
    append(s.main, [toolResult(sub.toolUseId)], `parent result for ${sub.agentId}`);
    await sleep(1500);
  }
  append(s.main, [endTurn(), stopHook()], "main end of turn");
  await sleep(2000);
}

async function multiScenario() {
  const ss = [0, 1, 2].map((n) => makeSession(n, `multi${n}`));
  ss.forEach((s, n) => {
    for (let i = 0; i < n; i++) spawnSub(s, i);
  });
  console.log("READY multi");
  for (let k = 0; k < 20; k++) {
    for (const s of ss) {
      const id = uid("toolu");
      append(s.main, [toolUse(id, "Grep", { pattern: "TODO", path: "src" })], `${s.sid} grep`);
      await sleep(700);
      append(s.main, [toolResult(id)], `${s.sid} grep result`);
    }
    await sleep(1000);
  }
}

async function soakScenario() {
  const minutes = Number(args.minutes ?? 10);
  const rate = Number(args.rate ?? 20);
  const s = makeSession(0, "soak");
  const sub = spawnSub(s, 0);
  console.log("READY soak");
  const end = Date.now() + minutes * 60000;
  const big = "x".repeat(50 * 1024);
  let n = 0;
  while (Date.now() < end) {
    const tickStart = Date.now();
    const batch = [];
    for (let i = 0; i < Math.ceil(rate / 2); i++) {
      const id = uid("toolu");
      batch.push(toolUse(id, n % 2 ? "Read" : "Bash", n % 2 ? { file_path: `src/f${n}.ts` } : { command: `echo ${n}` }));
      batch.push(toolResult(id, Math.random() < 0.05 ? big : "ok"));
      n++;
    }
    // `rate` lines per second: each iteration above writes 2 lines, so halve the pairs
    fs.appendFileSync(n % 3 === 0 ? sub.file : s.main, batch.slice(0, rate).join("\n") + "\n");
    if (n % (rate * 5) < rate) log(`soak batch n=${n}`);
    await sleep(Math.max(0, 1000 - (Date.now() - tickStart)));
  }
  append(s.main, [endTurn(), stopHook()], "soak end");
}

const hold = async (seconds) => {
  // keep the process (and its registry files) alive
  await sleep(seconds * 1000);
};

/** Session A: software architect, 2 coders, product spec, content writer; B and C are main-only. */
async function multiTeamScenario() {
  const [a, b, c] = [0, 1, 2].map((n) => makeSession(n, `team${n}`));
  const arch = ["gaisser-agents:software-architect", "Plan the change"];
  const coder = ["gaisser-agents:software-engineer", "Implement the plan"];
  spawnSub(a, 0, { role: arch });
  spawnSub(a, 1, { role: coder });
  spawnSub(a, 2, { role: coder });
  spawnSub(a, 3, { role: ["gaisser-agents:product-spec", "Write the spec"] });
  spawnSub(a, 4, { role: ["gaisser-agents:content-writer", "Draft the post"] });
  append(a.main, [toolUse(uid("toolu"), "Bash", { command: "npm test" })], "A main running");
  append(b.main, [endTurn(), stopHook()], "B main idle");
  append(c.main, [toolUse(uid("toolu"), "Grep", { pattern: "TODO" })], "C main grep");
  console.log("READY multi-team");
  await hold(Number(args.seconds ?? 600));
}

/** Main spawns an architect, which later returns to main (spawn walk, then handback walk). */
async function walkScenario() {
  const s = makeSession(0, "walk");
  append(s.main, [toolUse(uid("toolu"), "Bash", { command: "npm test" })], "main running");
  console.log("READY walk");
  await sleep(10000);
  for (let k = 0; k < 3; k++) {
    const sub = spawnSub(s, 0, { role: ["gaisser-agents:software-architect", "Plan the change"] });
    await sleep(10000);
    append(s.main, [toolResult(sub.toolUseId)], `parent result for ${sub.agentId}`);
    await sleep(10000);
  }
  await hold(Number(args.seconds ?? 60));
}

/** Product spec returns, then main spawns a software architect within 60 s (cross-floor relay). */
async function relayScenario() {
  const s = makeSession(0, "relay");
  append(s.main, [toolUse(uid("toolu"), "Bash", { command: "npm test" })], "main running");
  console.log("READY relay");
  await sleep(10000);
  for (let k = 0; k < 2; k++) {
    const p = spawnSub(s, 0, { role: ["gaisser-agents:product-spec", "Write the spec"] });
    await sleep(8000);
    append(s.main, [toolResult(p.toolUseId)], `product returned ${p.agentId}`);
    await sleep(6000);
    const a = spawnSub(s, 1, { role: ["gaisser-agents:software-architect", "Plan the change"] });
    await sleep(10000);
    append(s.main, [toolResult(a.toolUseId)], `architect returned ${a.agentId}`);
    await sleep(12000);
  }
  await hold(Number(args.seconds ?? 60));
}

/** 12 people on the Software floor: 4 coders plus one of each of 8 other roles. */
async function overflowScenario() {
  const s = makeSession(0, "overflow");
  append(s.main, [toolUse(uid("toolu"), "Bash", { command: "npm test" })], "main running");
  const roles = ["software-engineer", "software-engineer", "software-engineer", "software-engineer", "software-architect", "software-test-engineer", "software-debugger", "software-code-reviewer", "software-security-reviewer", "software-performance-reviewer", "software-db-engineer", "software-devops"];
  roles.forEach((r, i) => spawnSub(s, i, { role: [`gaisser-agents:${r}`, `Task ${i}`] }));
  console.log("READY overflow");
  await hold(Number(args.seconds ?? 600));
}

switch (scenario) {
  case "multi-team":
    await multiTeamScenario();
    break;
  case "walk":
    await walkScenario();
    break;
  case "relay":
    await relayScenario();
    break;
  case "overflow":
    await overflowScenario();
    break;
  case "static":
    await staticScenario();
    break;
  case "basic": {
    const s = makeSession(0, "basic");
    console.log("READY basic");
    await basicScenario(s, Number(args.seconds ?? 30));
    break;
  }
  case "spawn":
    await spawnScenario();
    break;
  case "multi":
    await multiScenario();
    break;
  case "soak":
    await soakScenario();
    break;
  default:
    console.error(`unknown scenario: ${scenario}`);
    process.exit(2);
}

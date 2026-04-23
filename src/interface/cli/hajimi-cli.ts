#!/usr/bin/env node
/**
 * Hajimi V3 CLI — Local-first AI agent command interface
 * Entry point for tool-system, agent lifecycle, memory, and ADR subcommands.
 */

import { Command } from "commander";
import { registerToolCommands } from "./commands/tools.js";
import { registerAgentCommands } from "./commands/agent.js";
import { registerMemoryCommands } from "./commands/memory.js";
import { registerAdrCommands } from "./commands/adr.js";

const program = new Command();

program
  .name("hajimi")
  .description("Hajimi V3 — Local-first AI agent CLI")
  .version("3.2.0")
  .configureOutput({ outputError: (str, write) => write(`\x1b[31mError:\x1b[0m ${str}`) });

registerToolCommands(program);
registerAgentCommands(program);
registerMemoryCommands(program);
registerAdrCommands(program);

// Unknown command handling
program.on("command:unknown", (cmd) => {
  console.error(`Unknown command: ${cmd[0]}`);
  program.help();
});

program.parseAsync(process.argv).catch((err) => {
  console.error(err instanceof Error ? err.message : String(err));
  process.exit(1);
});

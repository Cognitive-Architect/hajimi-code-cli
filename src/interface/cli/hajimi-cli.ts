#!/usr/bin/env node
import { Command } from "commander";
import { registerToolCommands } from "./commands/tools.js";
import { registerAdrCommands } from "./commands/adr.js";

const program = new Command();

program
  .name("hajimi")
  .description("Hajimi — Local-first AI agent CLI")
  .version("3.2.0");

registerToolCommands(program);
registerAdrCommands(program);

program.on("command:unknown", (cmd) => {
  console.error(`Unknown command: ${cmd[0]}`);
  program.help();
});

program.parseAsync(process.argv).catch((err) => {
  console.error(err instanceof Error ? err.message : String(err));
  process.exit(1);
});

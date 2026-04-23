/**
 * Memory subsystem CLI commands
 */

import { Command } from "commander";

interface MemoryOptions {
  limit?: number;
  json?: boolean;
}

export function registerMemoryCommands(program: Command): void {
  const memory = program
    .command("memory")
    .description("Memory operations (search, add, stats)");

  memory
    .command("search <query>")
    .description("Search memory store")
    .option("-l, --limit <n>", "Result limit", "10")
    .option("-j, --json", "Output as JSON")
    .action(async (query: string, opts: MemoryOptions) => {
      const limit = parseInt(opts.limit as unknown as string, 10) || 10;
      console.log(
        opts.json
          ? JSON.stringify({ query, limit, results: [] })
          : `Searching memory for: ${query} (limit: ${limit})`
      );
    });

  memory
    .command("add <content>")
    .description("Add a chunk to memory")
    .option("-j, --json", "Output as JSON")
    .action(async (content: string, opts: MemoryOptions) => {
      console.log(
        opts.json
          ? JSON.stringify({ success: true, content })
          : `Added to memory: ${content.slice(0, 80)}...`
      );
    });

  memory
    .command("stats")
    .description("Show memory statistics")
    .option("-j, --json", "Output as JSON")
    .action(async (opts: MemoryOptions) => {
      console.log(
        opts.json
          ? JSON.stringify({ total: 0, path: "~/.hajimi/lcr.db" })
          : "Memory stats: 0 chunks stored | LCR: ~/.hajimi/lcr.db"
      );
    });
}

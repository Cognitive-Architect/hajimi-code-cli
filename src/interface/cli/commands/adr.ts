/**
 * Knowledge / ADR CLI commands
 */

import { Command } from "commander";
import { readdir, readFile } from "fs/promises";
import { join } from "path";

interface AdrOptions {
  json?: boolean;
  limit?: number;
}

const ADR_DIR = "docs/adr";

async function listAdrs(): Promise<string[]> {
  try {
    const files = await readdir(ADR_DIR);
    return files.filter((f) => f.endsWith(".md")).sort();
  } catch {
    return [];
  }
}

export function registerAdrCommands(program: Command): void {
  const adr = program
    .command("adr")
    .description("Knowledge ADR commands — adr search, adr list");

  adr
    .command("search <keyword>")
    .description("Search ADR documents by keyword")
    .option("-l, --limit <n>", "Result limit", "10")
    .option("-j, --json", "Output as JSON")
    .action(async (keyword: string, opts: AdrOptions) => {
      const limit = parseInt(opts.limit as unknown as string, 10) || 10;
      const files = await listAdrs();
      const matches: { file: string; title: string }[] = [];
      for (const file of files.slice(0, limit)) {
        try {
          const content = await readFile(join(ADR_DIR, file), "utf-8");
          const title = content.split("\n")[0]?.replace("# ", "") || file;
          if (content.toLowerCase().includes(keyword.toLowerCase())) {
            matches.push({ file, title });
          }
        } catch { /* skip unreadable */ }
      }
      if (opts.json) {
        console.log(JSON.stringify({ keyword, matches }, null, 2));
      } else {
        if (matches.length === 0) {
          console.log(`No ADRs matched '${keyword}'.`);
        } else {
          console.log(`ADR matches for '${keyword}':`);
          matches.forEach((m) => console.log(`  - ${m.file}: ${m.title}`));
        }
      }
    });

  adr
    .command("list")
    .description("List all ADR documents")
    .option("-j, --json", "Output as JSON")
    .action(async (opts: AdrOptions) => {
      const files = await listAdrs();
      if (opts.json) {
        console.log(JSON.stringify({ count: files.length, files }, null, 2));
      } else {
        console.log(`Total ADRs: ${files.length}`);
        files.forEach((f) => console.log(`  - ${f}`));
      }
    });
}

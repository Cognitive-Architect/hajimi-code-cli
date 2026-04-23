/**
 * Tool-System CLI commands
 * Bridges Hajimi backend tools (Rust) to the CLI surface.
 */

import { Command } from "commander";
import { readFile, access } from "fs/promises";
import { spawn } from "child_process";
import { constants } from "fs";

interface ToolOptions {
  json?: boolean;
  limit?: number;
  pattern?: string;
}

async function runShell(cmd: string, args: string[]): Promise<{ stdout: string; stderr: string; code: number }> {
  return new Promise((resolve) => {
    const proc = spawn(cmd, args, { stdio: ["ignore", "pipe", "pipe"] });
    let stdout = "";
    let stderr = "";
    proc.stdout?.on("data", (d) => { stdout += d; });
    proc.stderr?.on("data", (d) => { stderr += d; });
    proc.on("close", (code) => resolve({ stdout, stderr, code: code ?? 1 }));
  });
}

function formatOutput(data: unknown, json: boolean): string {
  return json ? JSON.stringify(data, null, 2) : String(data);
}

export function registerToolCommands(program: Command): void {
  const tool = program
    .command("tool")
    .description("Run tool-system commands (read-file, grep, git-status, run-tests, build, security-audit)");

  tool
    .command("read-file <path>")
    .description("Read contents of a file")
    .option("-j, --json", "Output as JSON")
    .action(async (filePath: string, opts: ToolOptions) => {
      try {
        await access(filePath, constants.R_OK);
        const content = await readFile(filePath, "utf-8");
        console.log(formatOutput({ path: filePath, content }, opts.json ?? false));
      } catch (err) {
        console.error(`Error: cannot read file '${filePath}'`);
        process.exit(1);
      }
    });

  tool
    .command("grep <pattern> [paths...]")
    .description("Search for pattern in files")
    .option("-j, --json", "Output as JSON")
    .action(async (pattern: string, paths: string[], opts: ToolOptions) => {
      const targets = paths.length ? paths : ["."];
      const { stdout, stderr, code } = await runShell("grep", ["-rn", "--color=never", pattern, ...targets]);
      if (code !== 0 && !stdout) {
        console.error(stderr || `Pattern '${pattern}' not found`);
        process.exit(1);
      }
      console.log(formatOutput({ pattern, matches: stdout.split("\n").filter(Boolean) }, opts.json ?? false));
    });

  tool
    .command("git-status")
    .description("Show git repository status")
    .option("-j, --json", "Output as JSON")
    .action(async (opts: ToolOptions) => {
      const { stdout, stderr, code } = await runShell("git", ["status", "--short"]);
      if (code !== 0) {
        console.error(stderr || "Not a git repository");
        process.exit(1);
      }
      console.log(formatOutput({ status: stdout || "clean" }, opts.json ?? false));
    });

  tool
    .command("run-tests [crate]")
    .description("Run Rust test suite (cargo test)")
    .option("-j, --json", "Output as JSON")
    .action(async (crate: string | undefined, opts: ToolOptions) => {
      const args = crate ? ["test", "-p", crate] : ["test"];
      const { stdout, stderr, code } = await runShell("cargo", args);
      const summary = stdout.match(/test result:.*?\n/)?.[0]?.trim() || "unknown";
      console.log(formatOutput({ summary, stdout, stderr }, opts.json ?? false));
      if (code !== 0) process.exit(code);
    });

  tool
    .command("build")
    .description("Build the workspace (cargo build)")
    .option("-r, --release", "Release build")
    .action(async (opts: { release?: boolean }) => {
      const args = opts.release ? ["build", "--release"] : ["build"];
      const { stdout, stderr, code } = await runShell("cargo", args);
      console.log(stdout || stderr);
      if (code !== 0) process.exit(code);
    });

  tool
    .command("security-audit")
    .description("Run security audit (cargo audit)")
    .action(async () => {
      const { stdout, stderr, code } = await runShell("cargo", ["audit"]);
      console.log(stdout || stderr);
      if (code !== 0) process.exit(code);
    });
}

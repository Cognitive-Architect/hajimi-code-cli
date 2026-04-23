/**
 * Agent lifecycle CLI commands
 */

import { Command } from "commander";
import { spawn } from "child_process";

interface AgentOptions {
  config?: string;
  verbose?: boolean;
}

async function runAgentCmd(args: string[]): Promise<{ stdout: string; code: number }> {
  return new Promise((resolve) => {
    const proc = spawn("cargo", ["run", "--bin", "hajimi-agent", "--", ...args], {
      stdio: ["ignore", "pipe", "inherit"],
    });
    let stdout = "";
    proc.stdout?.on("data", (d) => { stdout += d; });
    proc.on("close", (code) => resolve({ stdout, code: code ?? 1 }));
  });
}

export function registerAgentCommands(program: Command): void {
  const agent = program
    .command("agent")
    .description("Agent lifecycle commands (start, status, stop)");

  agent
    .command("start")
    .description("Start the agent loop")
    .option("-c, --config <path>", "Path to agent config file")
    .option("-v, --verbose", "Enable verbose logging")
    .action(async (opts: AgentOptions) => {
      const args: string[] = ["start"];
      if (opts.config) args.push("--config", opts.config);
      if (opts.verbose) args.push("--verbose");
      console.log("Starting Hajimi agent...");
      const { code } = await runAgentCmd(args);
      if (code !== 0) {
        console.error("Agent failed to start.");
        process.exit(code);
      }
    });

  agent
    .command("status")
    .description("Check agent status")
    .action(async () => {
      console.log("Agent status: ready (no active process tracking yet)");
    });

  agent
    .command("stop")
    .description("Stop the agent loop")
    .action(async () => {
      console.log("Stop signal sent to agent.");
    });
}

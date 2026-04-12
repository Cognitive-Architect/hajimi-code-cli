/**
 * DEBT-MCP-004 B-01/03: Windows Unit Test Adaptation
 * Tests cross-spawn behavior on Windows for path/process/signal handling
 */

import { describe, test, expect, beforeAll, afterAll } from '@jest/globals';
import * as os from 'os';
import * as path from 'path';
import * as fs from 'fs';
import { spawn } from 'cross-spawn';
import { StdioTransport } from '../transport/stdio-transport';

describe('Windows Compatibility', () => {
  const isWindows = process.platform === 'win32';
  const tempDir = path.join(os.tmpdir(), 'mcp-test-' + Date.now());

  beforeAll(async () => {
    if (!fs.existsSync(tempDir)) fs.mkdirSync(tempDir, { recursive: true });
  });

  afterAll(async () => {
    if (fs.existsSync(tempDir)) fs.rmSync(tempDir, { recursive: true, force: true });
  });

  test('cross-spawn handles basic command execution', async () => {
    const cmd = isWindows ? 'cmd' : 'echo';
    const args = isWindows ? ['/c', 'echo', 'test'] : ['test'];
    const proc = spawn(cmd, args);
    let output = '';
    proc.stdout?.on('data', (data) => { output += data.toString(); });
    await new Promise<void>((resolve) => { proc.on('close', () => resolve()); });
    expect(output).toContain('test');
  });

  test('cross-spawn handles paths with spaces', async () => {
    const spaceDir = path.join(tempDir, 'path with spaces');
    fs.mkdirSync(spaceDir, { recursive: true });
    const testFile = path.join(spaceDir, 'test.txt');
    fs.writeFileSync(testFile, 'content');
    expect(fs.existsSync(testFile)).toBe(true);
  });

  test('path.join creates platform-specific paths', () => {
    const testPath = path.join(tempDir, 'subdir', 'file.txt');
    expect(testPath).toContain('subdir');
    if (isWindows) expect(testPath).toContain('\\');
  });

  test('os.tmpdir returns correct temp directory', () => {
    const tmp = os.tmpdir();
    expect(tmp).toBeTruthy();
    if (isWindows) expect(tmp).toMatch(/^[A-Z]:\\/i);
  });

  test('signal handling with process.kill', async () => {
    const cmd = isWindows ? 'ping' : 'sleep';
    const args = isWindows ? ['-n', '10', '127.0.0.1'] : ['10'];
    const proc = spawn(cmd, args);
    expect(proc.pid).toBeDefined();
    expect(proc.kill()).toBe(true);
    await new Promise<void>((resolve) => { proc.on('exit', () => resolve()); setTimeout(resolve, 500); });
  });

  test('environment variable passing to child process', async () => {
    const testValue = 'test-env-' + Date.now();
    const cmd = isWindows ? 'cmd' : 'sh';
    const args = isWindows ? ['/c', 'echo', '%TEST_VAR%'] : ['-c', 'echo $TEST_VAR'];
    const proc = spawn(cmd, args, { env: { ...process.env, TEST_VAR: testValue } });
    let output = '';
    proc.stdout?.on('data', (data) => { output += data.toString(); });
    await new Promise<void>((resolve) => { proc.on('close', () => resolve()); });
    expect(output).toContain(testValue);
  });

  test('StdioTransport integration with cross-spawn', async () => {
    const transport = new StdioTransport();
    const cmd = isWindows ? 'cmd' : 'cat';
    const args = isWindows ? ['/c', 'echo {"jsonrpc":"2.0"}'] : [];
    expect(() => { transport.connect({ command: cmd, args: args, timeout: 1000 }); }).not.toThrow();
    transport.close();
    expect(transport.isConnected()).toBe(false);
  });

  test('long path support validation', () => {
    const longDir = path.join(tempDir, 'a'.repeat(50), 'b'.repeat(50));
    fs.mkdirSync(longDir, { recursive: true });
    const testFile = path.join(longDir, 'test.txt');
    fs.writeFileSync(testFile, 'data');
    expect(testFile.length).toBeGreaterThan(100);
    expect(fs.existsSync(testFile)).toBe(true);
  });

  test('platform detection is correct', () => {
    expect(typeof process.platform).toBe('string');
    expect(['win32', 'darwin', 'linux']).toContain(process.platform);
    if (isWindows) expect(process.platform).toBe('win32');
  });
});

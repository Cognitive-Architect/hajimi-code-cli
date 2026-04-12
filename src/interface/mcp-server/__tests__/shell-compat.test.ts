import { describe, test, expect } from '@jest/globals';
import { spawn } from 'cross-spawn';

function runShell(cmd: string, args: string[], opts?: { shell?: boolean; windowsHide?: boolean }) {
  return new Promise<{ code: number; stdout: string; stderr: string }>((resolve) => {
    const child = spawn(cmd, args, opts);
    let stdout = '';
    let stderr = '';
    child.stdout?.on('data', (d: Buffer) => { stdout += d.toString(); });
    child.stderr?.on('data', (d: Buffer) => { stderr += d.toString(); });
    child.on('close', (code: number | null) => { resolve({ code: code ?? 0, stdout, stderr }); });
  });
}

describe('Shell Compatibility', () => {
  const isWin = process.platform === 'win32';

  describe('PowerShell', () => {
    test('ExecutionPolicy Bypass', async () => {
      if (!isWin) return;
      const r = await runShell('powershell', ['-ExecutionPolicy', 'Bypass', '-Command', 'Write-Host test']);
      expect(r.stdout).toContain('test');
    });

    test('Write-Host output capture', async () => {
      if (!isWin) return;
      const r = await runShell('powershell', ['-Command', 'Write-Host captured']);
      expect(r.stdout).toContain('captured');
    });

    test('nested quotes handling', async () => {
      if (!isWin) return;
      const r = await runShell('powershell', ['-Command', 'Write-Host "nested \"quotes\""']);
      expect(r.stdout).toContain('nested');
    });

    test('LASTEXITCODE propagation', async () => {
      if (!isWin) return;
      const r = await runShell('powershell', ['-Command', 'exit 42']);
      expect(r.code).toBe(42);
    });
  });

  describe('cmd.exe', () => {
    test('cmd /c command execution', async () => {
      if (!isWin) return;
      const r = await runShell('cmd', ['/c', 'echo cmdtest']);
      expect(r.stdout).toContain('cmdtest');
    });

    test('special characters escape', async () => {
      if (!isWin) return;
      const r = await runShell('cmd', ['/c', 'echo test ^& ^| ^< ^>']);
      expect(r.stdout).toContain('test');
    });

    test('exit code %ERRORLEVEL%', async () => {
      if (!isWin) return;
      const r = await runShell('cmd', ['/c', 'exit 7']);
      expect(r.code).toBe(7);
    });

    test('long command line near 8191 limit', async () => {
      if (!isWin) return;
      const longArg = 'x'.repeat(8000);
      const r = await runShell('cmd', ['/c', `echo ${longArg}`]);
      expect(r.stdout.length).toBeGreaterThan(7000);
    });
  });

  describe('cross-spawn shell options', () => {
    test('shell: true uses default shell', async () => {
      const r = await runShell('node', ['-e', 'console.log("shelltrue")'], { shell: true });
      expect(r.stdout).toContain('shelltrue');
    });

    test('windowsHide hides console', async () => {
      if (!isWin) return;
      const r = await runShell('cmd', ['/c', 'echo hidden'], { windowsHide: true, shell: true });
      expect(r.stdout).toContain('hidden');
      expect(r.code).toBe(0);
    });

    test('pwsh fallback', async () => {
      if (!isWin) return;
      const r = await runShell('pwsh', ['-Command', 'Write-Host pwshworks']);
      expect(r.stdout).toContain('pwshworks');
    });

    test('combined shell options', async () => {
      if (!isWin) return;
      const r = await runShell('powershell', ['-Command', 'Write-Host combined'], { shell: true, windowsHide: true });
      expect(r.stdout).toContain('combined');
    });
  });
});

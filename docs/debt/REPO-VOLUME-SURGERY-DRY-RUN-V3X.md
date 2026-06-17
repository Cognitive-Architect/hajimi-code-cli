# STONE-AUDIT-V3X-DAY13｜Repo Volume Surgery Dry Run

## 0. Scope

- Task: `STONE-AUDIT-V3X-DAY13 Repo Volume Surgery Dry Run + Explicit Execution Gate`
- Date: `2026-06-17`
- Branch: `stone-audit-v3x-controlled-demolition`
- HEAD before: `d51182f105a0ef0effbbd31dc4e05f75c9b125fd`
- Mode: docs-only dry run
- Production code changed: `NO`
- History rewrite executed: `NO`
- Models/target deleted: `NO`
- LFS initialized or changed: `NO`

Human summary: this document weighs the repository and lists surgery options. It does not perform surgery.

## 1. Git Coordinate Receipt

Command:

```powershell
git branch --show-current
git rev-parse HEAD
git status --short
```

Observed:

```text
branch: stone-audit-v3x-controlled-demolition
HEAD: d51182f105a0ef0effbbd31dc4e05f75c9b125fd
git status --short: clean
old dirty files staged: NO
```

## 2. Pack Size Receipt

Command:

```powershell
git count-objects -vH
```

Observed:

```text
count: 7403
size: 52.15 MiB
in-pack: 25411
packs: 2
size-pack: 3.81 GiB
prune-packable: 2910
garbage: 0
size-garbage: 0 bytes
```

Conclusion:

- Current measured pack size is `3.81 GiB`.
- The previous rebase baseline remains true.
- No improvement is claimed in this dry run.

## 3. Tracked Models / Target Receipt

Command:

```powershell
git ls-files models target src/interface/desktop/target
```

Observed tracked files:

```text
models/all-MiniLM-L6-v2.tar.gz
models/fast-all-MiniLM-L6-v2/config.json
models/fast-all-MiniLM-L6-v2/model.onnx
models/fast-all-MiniLM-L6-v2/special_tokens_map.json
models/fast-all-MiniLM-L6-v2/tokenizer.json
models/fast-all-MiniLM-L6-v2/tokenizer_config.json
models/fast-all-MiniLM-L6-v2/vocab.txt
```

Summary:

| Area | Result |
|---|---|
| tracked models | `7` files |
| tracked target | `0` files in current tree |
| target objects in history | `YES`, found by object scan |

## 4. Current Tracked Large Files

Command:

```powershell
$files = git ls-files
$rows = foreach ($f in $files) {
  if (Test-Path -LiteralPath $f -PathType Leaf) {
    $item = Get-Item -LiteralPath $f
    [PSCustomObject]@{ SizeBytes = $item.Length; SizeMiB = [Math]::Round($item.Length / 1MB, 2); Path = $f }
  }
}
$rows | Sort-Object SizeBytes -Descending | Select-Object -First 30
```

Top current tracked candidates:

| Rank | Size MiB | Path |
|---:|---:|---|
| 1 | 86.20 | `models/fast-all-MiniLM-L6-v2/model.onnx` |
| 2 | 79.33 | `models/all-MiniLM-L6-v2.tar.gz` |
| 3 | 13.67 | `src/intelligence/codex-twist/index.node` |
| 4 | 0.68 | `models/fast-all-MiniLM-L6-v2/tokenizer.json` |
| 5 | 0.33 | `src/patches/zstd-sys/zstd/lib/compress/zstd_compress.c` |
| 6 | 0.25 | `src/patches/zstd-sys/zstd/lib/common/xxhash.h` |
| 7 | 0.22 | `Cargo.lock` |
| 8 | 0.22 | `src/interface/web/app.js` |
| 9 | 0.22 | `models/fast-all-MiniLM-L6-v2/vocab.txt` |
| 10 | 0.22 | `docs/security/examples/SECURITY_REVIEW_SAMPLE.json` |

Observation:

- Current tree large-file pressure is concentrated in `models/`.
- `src/intelligence/codex-twist/index.node` is also a notable binary candidate, but it is far smaller than the model files.

## 5. Historical Large Object Scan

Command:

```powershell
git rev-list --objects --all |
  git cat-file --batch-check='%(objecttype) %(objectname) %(objectsize) %(rest)' |
  Where-Object { $_ -like 'blob *' } |
  ForEach-Object {
    $parts = $_ -split ' ', 4
    [PSCustomObject]@{
      SizeBytes = [Int64]$parts[2]
      SizeMiB = [Math]::Round(([Int64]$parts[2]) / 1MB, 2)
      Object = $parts[1]
      Path = if ($parts.Count -ge 4) { $parts[3] } else { '' }
    }
  } |
  Sort-Object SizeBytes -Descending |
  Select-Object -First 30
```

Top historical candidates:

| Rank | Size MiB | Object | Path |
|---:|---:|---|---|
| 1 | 100.00 | `102f878de8a7fdb386d3a009aa72cc7082b957e7` | `src/foundation/tests/fixtures/100mb.bin` |
| 2 | 86.20 | `4ca0f8fd1c947825610fd38f4ed63a144cb70738` | `models/fast-all-MiniLM-L6-v2/model.onnx` |
| 3 | 79.33 | `e41d5d8b206a3ea45b816327c6124e04096eb5e5` | `models/all-MiniLM-L6-v2.tar.gz` |
| 4 | 43.47 | `813546769b177b0e15e03f2c9396d261d96a8e3b` | `target/debug/deps/libtokio-b00e9910bd6a0062.rlib` |
| 5 | 43.09 | `0f61d35fab0f45a516ed8102e8e2074837e0ed92` | `target/debug/deps/libtokio-8b0094535baebc5c.rlib` |
| 6 | 35.35 | `0c293e92b9efa6247e1233e727158eb601dc1d99` | `target/debug/deps/codex_twist-d94a163dccd8a90f.pdb` |
| 7 | 31.86 | `8d19cb3cb0c56edf61b2616d296d1c8f35648985` | `crates/hajimi-hnsw/target/debug/deps/libsyn-ee55c1d2e7d8212...` |
| 8 | 31.23 | `639881e0a45f3e7fc805bb51ae1ab7af74a56de8` | `crates/hajimi-hnsw/target/debug/deps/libwasm_bindgen_macro_...` |
| 9 | 30.96 | `be504209fd23f9b81414bf8be8cdfc8922f9c693` | `target/debug/deps/libsyn-c39e05a3aca33b05.rlib` |
| 10 | 30.59 | `7a87d8c5c8d71764839235e9979afa189e4370a8` | `target/debug/deps/codex_twist-c6e543ebc1067614.pdb` |

Observation:

- Historical weight is not only from current model files.
- Previous tracked build artifacts under `target/` and `crates/hajimi-hnsw/target/` are visible in history.
- The largest discovered object is `src/foundation/tests/fixtures/100mb.bin`.

## 6. Tool Availability

Commands:

```powershell
git lfs version
Get-Command git-filter-repo -ErrorAction SilentlyContinue
```

Observed:

```text
git lfs version: git-lfs/3.4.1 (GitHub; windows amd64; go 1.20.11; git 0898dcbc)
git-filter-repo: NOT AVAILABLE
```

Conclusion:

- Git LFS is available on this machine.
- `git-filter-repo` is not available as a command.
- No dependency was installed for this dry run.

## 7. Candidate Action Table

| Candidate | What it would do | Risk | Approval required | Dry-run recommendation |
|---|---|---|---|---|
| Keep current history | Do nothing to history | Pack remains `3.81 GiB` | No | Safe default |
| Add or verify ignore rules for build output | Prevent future `target/` tracking | Low if only `.gitignore` changes | Yes, separate task | Recommended separate low-risk task |
| Move model files to LFS going forward | Store large models outside normal Git blobs | Requires workflow agreement; clone behavior changes | YES | Candidate after approval |
| Replace models with release assets/download script | Reduce current tree weight if models are not required in source | Runtime/dev onboarding may change | YES | Candidate after product/runtime decision |
| History rewrite to purge `target/`, `100mb.bin`, model blobs | Shrink pack history | Very high; rewrites every clone and branch relationship | YES, explicit | Do not execute without fresh backup/tag/clone plan |
| Investigate `src/intelligence/codex-twist/index.node` | Decide whether binary should stay tracked | Medium; may be required by runtime | YES, separate task | Sample first, do not remove here |

## 8. APPROVAL REQUIRED

The following actions require explicit user approval in a later task:

1. Running any `git filter-repo`, BFG, or equivalent history rewrite.
2. Deleting or moving any file under `models/`.
3. Deleting or rewriting any historical `target/` object.
4. Adding or changing `.gitattributes` for LFS.
5. Changing `.gitignore` if it is paired with tracked file cleanup.
6. Force-pushing rewritten history.
7. Asking collaborators to reclone or reset local branches.

No approval was granted in Day13, so none of these actions were performed.

## 9. Fresh Backup / Rollback Plan Before Any Future Rewrite

Before a future rewrite task can run, require:

1. Confirm a clean worktree:

```powershell
git status --short
git diff --cached --name-only
```

2. Create a fresh local safety tag:

```powershell
git tag stone-v3x-before-repo-volume-rewrite-YYYYMMDD-HHMMSS
git push origin stone-v3x-before-repo-volume-rewrite-YYYYMMDD-HHMMSS
```

3. Create a fresh clone outside the active repo and run the rewrite there.

4. Validate the rewritten clone:

```powershell
git count-objects -vH
cargo check -p hajimi-desktop
npm run test:security-gate
git log --oneline -n 5
```

5. Compare pack size before and after.

6. Only after explicit confirmation, force-push with lease:

```powershell
git push --force-with-lease origin stone-audit-v3x-controlled-demolition
```

7. Rollback path if anything fails:

```powershell
git push --force-with-lease origin stone-v3x-before-repo-volume-rewrite-YYYYMMDD-HHMMSS:stone-audit-v3x-controlled-demolition
```

## 10. Forbidden Diff Receipt

Commands:

```powershell
git diff --name-only -- src Cargo.toml package.json src/interface/desktop/tauri.conf.json
git diff --cached --name-only
git reflog -n 3
```

Observed:

```text
forbidden production diff: none
cached files before doc stage: none
reflog top: d51182f1 HEAD@{0}: commit: docs(roadmap): add Codex handoff for STONE-AUDIT-V3X v0.14
```

Conclusion:

- Production changes: `NO`
- Old dirty files staged: `NO`
- History rewrite executed: `NO`

## 11. Day14 Closure Inputs

Day14 can use this document as the repo-volume evidence source:

| Input | Value |
|---|---|
| pack size | `3.81 GiB` |
| tracked models | `7` files |
| tracked target | `0` current files |
| historical target objects | `YES` |
| LFS availability | `AVAILABLE` |
| git-filter-repo availability | `NOT AVAILABLE` |
| rewrite approval | `NOT GRANTED` |
| recommended next | decide whether to approve a separate repo-volume rewrite plan or close with known debt |

## 12. Self-Check Against Day13 Knife Table

| Check | Status | Evidence |
|---|---|---|
| FUNC-001 pack size recorded | `PASS` | `git count-objects -vH` |
| FUNC-002 models tracked list | `PASS` | `git ls-files models` |
| FUNC-003 target tracked list | `PASS` | `git ls-files target src/interface/desktop/target` |
| FUNC-004 large object analysis availability | `PASS/PARTIAL` | built-in `git rev-list`/`git cat-file`; `git-filter-repo` unavailable |
| CONST-001 no production diff | `PASS` | forbidden diff empty |
| CONST-002 no history rewrite | `PASS` | no rewrite command executed |
| CONST-003 no file deletion | `PASS` | status clean before doc |
| CONST-004 approval gate present | `PASS` | `APPROVAL REQUIRED` section |
| NEG-001 filter-repo not mutating | `PASS` | only availability check |
| NEG-002 LFS not initialized | `PASS` | no `.gitattributes` or LFS config changes |
| NEG-003 pack target not claimed met | `PASS` | measured `3.81 GiB`; no improvement claimed |
| NEG-004 old dirty unstaged | `PASS` | cached diff empty |
| UX-001 cleanup path readable | `PASS` | candidate table |
| UX-002 rollback/fresh clone readable | `PASS` | backup plan section |
| E2E-001 final closure inputs ready | `PASS` | Day14 input table |
| HIGH-001 history rewrite requires approval | `PASS` | approval gate |

## 13. Final Recommendation

Day13 should close as docs-only dry run.

Recommended next action:

- Do not rewrite history in this task.
- For Day14 final closure, record repo volume as `NOT REDUCED / APPROVAL REQUIRED`.
- If the user wants actual size reduction, open a separate high-risk task with explicit approval, fresh backup tag, fresh clone rewrite, validation, and force-with-lease gate.

You’ve made excellent progress. The core runtime/user issues are largely resolved, and your test infrastructure is a big step up. Below are targeted code-level findings, gaps to close, test additions to make green in CI, and a short list of UX/polish improvements. I’ll also point out a couple of spec alignments with PEP 723 and related packaging docs.

What’s solid now
- Real-time output and argument forwarding: Switching to inherited stdio and adding separator logic makes run behavior feel right.
- CWD policy and CLI overrides: The `CliOverrides` plumbing and `Executor::resolve_working_directory` are good.
- Offline and uv flags: Good precedence and dry-run transparency.
- Completions are implemented.
- Containerized integration tests: This will pay dividends for Linux and reproducibility.
- MCP path fix: Using `current_exe()` avoids the earlier debug-path trap.

Priority issues to fix next

1) Offline override isn’t applied to introspection
- In `IntrospectionRunner::run_introspection`, `UV_NO_NETWORK` is set only when `config.core.offline` is true. CLI `--offline` never modifies `config.core.offline`, and there’s no `PYST_OFFLINE` propagation.
- Fix: Accept an `offline_override: Option<bool>` in `IntrospectionRunner::new_with_overrides(...)` (parallel to `Executor`) and set `UV_NO_NETWORK=1` if either the override is true or config is true. Wire from CLI in `handle_list`, `handle_info`, `document`, and MCP paths.

2) Exit code mapping for offline
- Spec says use 102 for network-required failures. Currently `Executor` returns `GenericError` for any nonzero status.
- Low-effort heuristic: if offline mode is active (override/config) and uv exits nonzero, map to `ExitCode::NetworkRequired`. This errs on the safe side and makes CI behavior deterministic.

3) `--` consistency and dry-run output
- Runtime: You add `--` only when “needed” and uv flags are present. Dry-run prints the `--` unconditionally.
- Unify: Either always add `--` when there are script args (simplest and future-proof) or match dry-run to actual invocation logic. Right now it’s confusing to users inspecting dry runs.

4) Cache validity and Python version mismatch
- Cache checks use `python3 --version` in `Cache`, but introspection executes under `uv run` (potentially a different interpreter/managed version). That can cause stale or over-eager invalidation.
- Fix: Use the introspector’s reported `python_version` (already part of `IntrospectionResult`) as the canonical version in the cache entry. Or compute version by `uv run python --version` in `Cache` to match environments; best is to treat the `IntrospectionResult`’s `python_version` as source of truth.

5) Cache stats are broken
- `Cache::get_stats` calls `key_to_path`, which always returns an error. Your “valid_entries” will be 0 forever.
- Quick fix: Store the absolute script path in `CacheEntry` (add a `path` field) or use the path itself as the key. Then `get_stats` can validate without reverse-engineering.

6) Duplicate names: installed vs local
- `pyst list` merges installed scripts with discovered scripts without deduplication. You’ll show duplicates (and potentially run the “wrong one” vs precedence).
- Fix: Deduplicate by name using your precedence (local vs global) and collapse entries to a single “effective” script for the main listing. Provide `--all` to show both origins.

7) `introspector.py`: click decorator detection bug
- The check `decorator.id` for `ast.Attribute` won’t exist and is wrong. Either:
  - simplify to import-level detection (you already detect `import click`/`import typer`), or
  - handle `ast.Call` where `func` can be an `ast.Attribute` (`click.command`) or an `ast.Name` with proper stringification.

8) Robustness of helper paths
- `IntrospectionRunner::get_introspector_path` and `Documenter::find_documenter_helper` use relative paths near the binary. This is fragile under packaging or `cargo install`.
- Safer options:
  - Embed the helpers with `include_str!` and write them to a temp file on first use (and cache the temp path), or
  - Install helpers under the data dir (`get_data_dir()/helpers`) at first run and reference from there.

9) Unused flags
- `--color/--no-color`, `-v/--verbose`, `-q/--quiet` are parsed but unused.
- Either implement a minimal `tracing` setup with `RUST_LOG` mapping to these levels and use `console`/`yansi` for color, or remove until ready.

10) Tilde expansion via `dirs`
- `dirs::home_dir` is deprecated. Consider `directories-next` or the `home` crate to resolve the home dir reliably across platforms.

Tests to add or extend

Execution and args
- Always stream stdout/stderr test: already covered in containers; also cover native with a script printing interleaved stdout/stderr to catch buffering issues.
- Separator and quoting: Add a test where script args include edge cases (`--`, `-n`, quotes, spaces, unicode). Ensure the same output with and without uv flags. Include a Windows-only variant if possible.

Exit codes
- Exit bubbling: Write a `.pyst/test-exit-codes.py` (you have it) and assert `pyst run test-exit-codes 5` exits nonzero; decide policy (bubble exact code or map). If mapping, assert the mapping.
- Offline 102: Use `.pyst/test-offline.py` that imports a dependency not in cache and fails offline. Assert 102.

Contexts
- `--force`: Add a test where a script blocked by `!db-*` executes with `--force`, and without it exits 101. Also test `--context X` overrides env `PYST_CONTEXT`.

Discovery & selectors
- Collision: Create a local `hello` and install a global `hello` and test `project:hello`, `global:hello`, and precedence rules in `list` (dedup, single effective entry).
- Absolute path run: `pyst run ./relative/path.py` respects contexts (blocked unless `--force`), as per your uniformity rule.

Introspection and cache
- `--no-cache`: Toggle descriptions and ensure `list`/`info` reflect changes despite cache. Add a Python version change simulation if feasible (or inject a different introspector version into cache key).
- Click/Typer detection: Cover Import vs Safe mode (treat as future test if import mode will do more).

Install/update/uninstall
- Happy-path smoke: install from a raw URL/gist (using your `external_files` URLs), `list` shows it (dedup), `run` works, `uninstall` removes it from manifest and disk.

MCP
- Stdio handshake: Send `initialize` and `tools/list`; expect non-error responses. Optional: call `run_script` with a safe script and assert text content is returned.

Windows and spaces
- If you can, add a native Windows CI job that runs a basic `pyst run` with a script located in a path with spaces.

Spec alignment notes (PEP 723 and packaging)
- PEP 723 inline script metadata is stable; the block is `# /// script` with TOML inside including `requires-python` and `dependencies` keys. Your extractor matches this model, but consider using the published scanning approach (regex) to avoid corner cases. The user guide has a reference implementation and details about the block grammar [packaging.python.org](https://packaging.python.org/en/latest/specifications/inline-script-metadata/).
- Long term, consider deferring to uv for metadata parsing if possible (so you don’t diverge).
- If you later support discovery from wheels or installed packages via entry points, review the entry points spec (where `entry_points.txt` in `.dist-info` defines console scripts) [packaging.python.org](https://packaging.python.org/en/latest/specifications/entry-points/).
- More background on inline dependencies in scripts and practical ergonomics: [notes.myhro.info](https://notes.myhro.info/2024/10/inline-dependencies-in-python-scripts/).
- Related metadata references (if you start reading `pyproject.toml` or core metadata): [packaging.python.org](https://packaging.python.org/en/latest/specifications/core-metadata/) and [packaging.python.org](https://packaging.python.org/en/latest/specifications/section-distribution-metadata/).

What’s left from your plan (condensed checklist)
- Introspection
  - Batch mode (single `uv run` calling `introspector.py` for multiple files).
  - Import-mode enhancement for trusted scripts (actual runtime parameter introspection when safe).
- Contexts
  - `extends` inheritance and provenance in `explain` (file path + rule index).
- Install/update/shims
  - PATH shims in `~/.local/share/pyst/bin` (and Windows `.cmd/.ps1`) that route through `pyst run <name>` by default. Optional “direct uv” shim mode later.
- MCP
  - Return structured JSON in `result.content` (e.g., `{"type":"json","json":{...}}`) rather than only text; keeps clients happy.
  - TCP transport and basic auth later.
- Document
  - Provider abstraction beyond DSPy/OpenRouter; minimal guardrails are there, so this can wait.

UX/polish opportunities
- Diagnostics: `pyst doctor` or `pyst --diagnostics` to print active context, resolved cwd policy, uv flags, config/caches paths, and whether uv is on PATH.
- Clear errors
  - If `uv` is missing: show a one-liner with install instructions.
  - If offline fails: exit 102 with a concise hint about offline mode and how to pre-warm deps.
- Listing
  - `pyst list --disabled-only` to quickly audit blocked items.
  - Markdown output as a table for easier copy/paste into docs.
- Explain on block
  - When a run is blocked, show the final rule (pattern and context) inline, not just “use explain.”

Smaller code nits
- `Discovery::detect_entry_point`: harmless heuristics; just ensure this doesn’t regress performance. You can tighten later with AST for fewer false positives.
- Use `directories-next` or `home` to replace `dirs` for path resolution.
- Consider wrapping uv invocation failures to include the fully resolved command and the cwd (you already do this for dry-run; capturing it in errors aids support).

Overall assessment
- Core behavior is correct and feels snappy; your container tests cover the right surface.
- Fix the offline override propagation to the introspector, align exit code 102, and straighten the `--` dry-run mismatch. Then add the dedup in list-resolution and address the cache versioning mismatch. Those are the biggest correctness/UX wins remaining in Phase 1/2.
- Aligning PEP 723 parsing with the user guide’s approach ensures long-term compatibility [packaging.python.org](https://packaging.python.org/en/latest/specifications/inline-script-metadata/).

If you want, I can draft minimal patches for:
- Propagating CLI offline to `IntrospectionRunner`.
- Unifying `--` behavior and dry-run output.
- Fixing cache stats by storing `path` in `CacheEntry`.
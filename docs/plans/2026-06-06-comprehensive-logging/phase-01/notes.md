# Phase 1 - Decision Notes

## Task 1

### Decisions made (not in spec)
- Kept logging config as store model types only; the current app has snippet YAML loaders and prefs state, but no single app-wide YAML store object to load at startup.

### Spec deviations
- none

### Tradeoffs accepted
- `LoggingConfig` uses string levels to preserve the YAML contract and defer validation to `tracing_subscriber::EnvFilter` in task 4.

### Assumptions
- The later command/frontend phases can reuse `LoggingFrontendConfig` directly from the store model.

### Follow-ups for human
- none

### Test evidence
- RED: `rtk cargo test logging_config --test store_yaml` failed with unresolved import `openmacro_lib::store::LoggingConfig`.
- GREEN: `rtk cargo test logging_config --test store_yaml` passed 2 tests.

## Task 2

### Decisions made (not in spec)
- `RingBuffer::push` assigns the authoritative sequence number to each entry so tests and future callers cannot accidentally create duplicate `seq` values.

### Spec deviations
- none

### Tradeoffs accepted
- Event fields that only implement `Debug` are captured as strings; typed primitives are preserved as JSON numbers, booleans, or strings.

### Assumptions
- Sequence values starting at 1 satisfy the monotonic `seq` contract and keep `slice_since(0)` as the full-ring query.

### Follow-ups for human
- none

### Test evidence
- RED: `rtk cargo test --test log_init_ring` failed with `could not find log_init in openmacro_lib`.
- GREEN: `rtk cargo test --test log_init_ring` passed 2 tests.

## Task 3

### Decisions made (not in spec)
- `set_verbose_content` updates the atomic flag each time it is called so tests and startup init can exercise both config and env-derived values within one process.

### Spec deviations
- none

### Tradeoffs accepted
- Only `OPENMACRO_LOG_VERBOSE=1` overrides config to verbose; other env values fall back to the config flag.

### Assumptions
- Exporting `log_body!` at crate root is acceptable for future call sites because `#[macro_export]` macros are rooted there.

### Follow-ups for human
- none

### Test evidence
- RED: `rtk cargo test --test log_init_redact` failed with unresolved `log_body`, `log_init::redact`, `set_verbose_content`, and `verbose_content`.
- GREEN: `rtk cargo test --test log_init_redact` passed 6 tests.

## Task 4

### Decisions made (not in spec)
- `log_dir()` uses `dirs::config_dir().join("openmacro").join("logs")`, matching the existing prefs/sync/crash convention rather than a Tauri `app_data_dir` or `app_log_dir` API, because this codebase does not currently use Tauri app-dir helpers for backend storage paths.
- Added `OPENMACRO_LOG_DIR` as a test override for pruning and init tests.
- Corrected the shipped default YAML integration test expectation from 5 to 6 snippets after the full suite showed the tracked `snippets/default.yaml` currently includes six snippets.

### Spec deviations
- Runtime startup currently initializes logging with `LoggingConfig::default()` because there is no existing app-wide YAML store object with a `logging` field to load before Tauri setup.

### Tradeoffs accepted
- If the log directory cannot be created, init falls back to a sink-backed file guard instead of panicking during app startup.

### Assumptions
- Daily log filenames can be pruned by lexicographic filename order because the configured pattern is `openmacro.log.YYYY-MM-DD`.

### Follow-ups for human
- Wire startup logging to a real persisted app config if/when a single app-wide YAML store is introduced.

### Test evidence
- RED: `rtk cargo test --test log_init_filter --test log_init_rotation` failed with missing `build_env_filter_directive`, `init`, and `rotation`.
- GREEN: `rtk cargo test --test log_init_filter --test log_init_rotation` passed 5 tests.
- Full-suite RED: `rtk cargo test` failed only `shipped_default_yaml_loads_without_errors` because the tracked default YAML has 6 snippets while the test expected 5.

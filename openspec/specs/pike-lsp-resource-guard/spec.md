# pike-lsp-resource-guard Specification

## Purpose
TBD - created by archiving change lsp-resource-guard. Update Purpose after archive.
## Requirements
### Requirement: The LSP process enforces an RSS ceiling
The Pike LSP process SHALL monitor its own resident set size (RSS) and terminate itself when RSS exceeds the configured maximum. The default maximum SHALL be 256 MiB, and users SHALL be able to override it with `--max-rss-mb <MiB>` or `PIKE_LSP_MAX_RSS_MB=<MiB>`.

#### Scenario: Default guard is active
- **WHEN** `pike-lsp stdio` starts without a resource override
- **THEN** the process monitors RSS against a 256 MiB ceiling.

#### Scenario: Environment override changes the ceiling
- **WHEN** `PIKE_LSP_MAX_RSS_MB=128 pike-lsp stdio` starts
- **THEN** the process monitors RSS against a 128 MiB ceiling.

#### Scenario: CLI override wins over environment
- **WHEN** `PIKE_LSP_MAX_RSS_MB=128 pike-lsp --max-rss-mb 64 stdio` starts
- **THEN** the process monitors RSS against a 64 MiB ceiling.

#### Scenario: Zero disables the guard
- **WHEN** `pike-lsp --max-rss-mb 0 stdio` starts
- **THEN** the process does not spawn the RSS monitor.

### Requirement: Runaway RSS terminates the process cleanly
The Pike LSP process MUST write a diagnostic message to stderr and exit with code 137 when RSS exceeds the configured maximum. The diagnostic MUST include the measured RSS and the configured limit.

#### Scenario: RSS exceeds ceiling
- **WHEN** the RSS monitor observes process RSS greater than the configured ceiling
- **THEN** stderr contains a resource-guard diagnostic with both measured and limit values
- **AND** the process exits with code 137.

### Requirement: Unsupported RSS platforms fail open with a warning
The Pike LSP process SHALL treat unsupported RSS measurement platforms as guard-unavailable rather than crashing during startup. It MUST log a warning when a non-zero RSS limit is configured but RSS measurement is unavailable.

#### Scenario: RSS measurement unavailable
- **WHEN** the process starts on a platform where RSS cannot be measured
- **THEN** the LSP still starts
- **AND** stderr contains a warning that the RSS guard is unavailable.


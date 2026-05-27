# ansible-rust-lint-rust

A Rust reimplementation of [ansible-rust-lint](https://github.com/ansible/ansible-rust-lint) — CLI-compatible drop-in replacement for linting Ansible playbooks, task files, roles, and collections.

## Features

- Drop-in CLI replacement: same flags, same exit codes, same output formats
- 44 built-in rules across all upstream profiles
- 6 output formats: `brief`, `full`, `json`, `sarif`, `codeclimate`, `pep8`
- Auto-fix support (`--fix`) for formatting violations
- Inline `# noqa` suppression and `.ansible-rust-lint-ignore` file support
- Config loading from `.ansible-rust-lint`, `.ansible-rust-lint.yml`, and `.config/ansible-rust-lint.yml`
- Profile-based rule filtering (min → basic → moderate → safety → shared → production)

## Installation

```sh
cargo install --path crates/ansible-rust-lint
```

Or build from source:

```sh
cargo build --release
# Binary is at target/release/ansible-rust-lint
```

## Usage

```sh
# Lint the current directory
ansible-rust-lint

# Lint specific files or directories
ansible-rust-lint playbooks/ roles/myrole

# Use a specific profile
ansible-rust-lint --profile moderate

# Skip specific rules
ansible-rust-lint --skip-list yaml[truthy],name[missing]

# Output as JSON
ansible-rust-lint --format json

# Treat warnings as errors
ansible-rust-lint --strict

# Auto-fix all fixable violations
ansible-rust-lint --fix

# Auto-fix only specific rules
ansible-rust-lint --fix yaml[truthy],yaml[trailing-spaces]

# List all rules
ansible-rust-lint --list-rules

# List available profiles
ansible-rust-lint --list-profiles

# Generate an ignore file from current violations
ansible-rust-lint --generate-ignore > .ansible-rust-lint-ignore
```

## Rules

| ID | Severity | Description |
|----|----------|-------------|
| `yaml[truthy]` | warning | Truthy value should be true or false |
| `yaml[line-length]` | warning | Lines should not exceed the maximum line length |
| `yaml[trailing-spaces]` | warning | Lines must not have trailing whitespace |
| `yaml[indentation]` | error | YAML indentation must use spaces, not tabs |
| `yaml[empty-lines]` | warning | Too many blank lines |
| `yaml[document-start]` | warning | Missing document start marker `---` |
| `yaml[new-line-at-end-of-file]` | warning | File must end with a newline |
| `yaml[comments]` | warning | Comment must start with a space after `#` |
| `yaml[key-duplicates]` | error | Duplicate keys in YAML mappings are not allowed |
| `yaml[octal-values]` | error | Octal values must use 0o prefix |
| `yaml[brackets]` | warning | Too many spaces inside brackets |
| `yaml[colons]` | warning | Colons must be followed by a space |
| `yaml[commas]` | warning | Commas must be followed by a space |
| `name[missing]` | warning | All tasks should have a name |
| `name[casing]` | warning | Task names should start with an uppercase letter |
| `name[template]` | warning | Task names should not be a bare Jinja2 template |
| `fqcn[action]` | warning | Use FQCN for module names |
| `no-changed-when` | warning | Commands should not change things if nothing needs changing |
| `risky-shell-pipe` | error | Shells that use pipes without pipefail are risky |
| `command-instead-of-module` | warning | Avoid using command when a dedicated module is available |
| `command-instead-of-shell` | warning | Use command instead of shell when shell features are not needed |
| `no-log-password` | error | Tasks that deal with passwords must have no_log enabled |
| `var-naming[pattern]` | warning | Variables should use snake_case naming |
| `partial-become` | error | become_user requires become: true |
| `package-latest` | warning | Package installs should not use state: latest |
| `risky-file-permissions` | warning | File permissions should be specified explicitly |
| `deprecated-bare-vars` | warning | Variables in loop/with_items must use Jinja2 syntax |
| `no-free-form` | warning | Avoid free-form module arguments; use key/value mapping |
| `no-jinja-when` | error | `when:` conditions should not use `{{ }}` Jinja2 syntax |
| `ignore-errors` | warning | Avoid ignore_errors; use failed_when instead |
| `literal-compare` | warning | Use `true`/`false` not `True`/`False` in conditions |
| `empty-string-compare` | warning | Don't compare to empty string |
| `key-order[task]` | warning | The `name` key should come first in a task |
| `no-relative-paths` | warning | Avoid relative paths in roles |
| `only-builtins` | warning | Use only ansible.builtin modules for portable roles |
| `no-handler` | warning | Tasks that restart services should use notify/handlers |
| `role-name` | warning | Role names must use snake_case |
| `run-once[task]` | warning | run_once with delegate_to may have unexpected results |
| `load-failure[yaml]` | error | Failed to load or parse YAML file |
| `meta-no-info` | warning | Role meta/main.yml is missing required fields |
| `meta-incorrect` | warning | Role meta/main.yml should specify min_ansible_version |
| `meta-no-tags` | warning | Role meta/main.yml is missing galaxy_info.galaxy_tags |

### Auto-fixable rules

The `--fix` flag can automatically remediate:

| Rule | Fix applied |
|------|-------------|
| `yaml[truthy]` | Replaces `yes`/`no`/`on`/`off` with `true`/`false` |
| `yaml[trailing-spaces]` | Strips trailing whitespace from each line |
| `yaml[new-line-at-end-of-file]` | Adds a trailing newline |
| `yaml[document-start]` | Prepends `---` document start marker |

## Profiles

Profiles are cumulative — each includes all rules from the profiles below it.

| Profile | Description |
|---------|-------------|
| `min` | Mandatory rules that prevent fatal errors |
| `basic` | Common coding issues and style enforcement (default) |
| `moderate` | Best practices for maintainability |
| `safety` | Avoids non-deterministic or security-risky calls |
| `shared` | Additional strictness for shared collections |
| `production` | Maximum strictness for production workloads |

## Configuration

ansible-rust-lint-rust reads configuration from (searched upward from the working directory):

- `.ansible-rust-lint`
- `.ansible-rust-lint.yml` / `.ansible-rust-lint.yaml`
- `.config/ansible-rust-lint.yml` / `.config/ansible-rust-lint.yaml`

Example `.ansible-rust-lint.yml`:

```yaml
profile: moderate
skip_list:
  - yaml[line-length]
  - name[casing]
warn_list:
  - no-handler
exclude_paths:
  - vendor/
  - tests/
var_naming_pattern: "^[a-z_][a-z0-9_]*$"
```

### Ignoring violations inline

Add `# noqa` to suppress all rules on a line, or `# noqa: rule-id` for a specific rule:

```yaml
- name: Start service
  service:
    name: nginx
    state: started
  ignore_errors: true  # noqa: ignore-errors
```

### .ansible-rust-lint-ignore file

To suppress specific rule/file combinations:

```
# path/to/file.yml rule-id
roles/legacy/tasks/main.yml yaml[truthy]
playbooks/old.yml name[missing]
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | No violations (warnings do not count unless `--strict`) |
| 1 | Rule violations found |
| 2 | Fatal error (invalid config, parse failure, bad CLI arguments) |

## Development

```sh
# Run tests
cargo test --workspace

# Run lints
cargo clippy --workspace -- -D warnings

# Run coverage
cargo llvm-cov --workspace
```

### Project structure

```
crates/
  ansible-rust-lint/          # Binary crate (CLI entry point)
  ansible-rust-lint-core/     # Library crate
    src/
      config.rs          # Config loading and merging
      discovery.rs       # File discovery and classification
      fix.rs             # Auto-fix engine
      formatter/         # Output formatters (brief, full, json, sarif, ...)
      parser/            # YAML parser with position tracking, task/playbook model
      registry.rs        # Rule registry and profile filtering
      rule.rs            # Rule trait and core types
      rules/             # All built-in rule implementations
      runner.rs          # Main lint pipeline
```

## Scope and limitations

- Custom rule plugins are not supported
- `schema[*]` validation rules (require JSON Schema against Ansible schemas) are not implemented
- `galaxy[*]` rules (require Galaxy API access) are not implemented
- `syntax-check` (requires `ansible-playbook` binary) is not implemented

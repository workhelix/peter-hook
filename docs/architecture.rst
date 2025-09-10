Architecture
============

Core Components
---------------

- ``src/config/parser.rs``: Parse and validate ``hooks.toml``; supports string/array commands and execution strategies
- ``src/hooks/resolver.rs``: Find nearest config; resolve groups; apply file targeting
- ``src/hooks/dependencies.rs``: Topological sort and phase planning
- ``src/hooks/executor.rs``: Execute hooks with sequential/parallel/force-parallel strategies; enforce safety
- ``src/config/templating.rs``: Resolve template variables in commands, env, and workdir
- ``src/git/installer.rs``: Install/uninstall managed git hooks; list supported events
- ``src/git/changes.rs``: Detect changed files for working directory, push, or commit ranges; glob matching
- ``src/cli/mod.rs``: Clap-based CLI definition

Execution Model
---------------

- Resolve hooks for the requested event from the nearest ``hooks.toml``
- If ``--files`` is used, compute changed files and filter hooks by patterns
- If dependencies exist, build an execution plan with parallel phases
- Execute hooks, running read-only hooks in parallel and repository-modifying hooks sequentially
- Collect exit codes and print a concise summary

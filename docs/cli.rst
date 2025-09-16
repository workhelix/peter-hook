CLI Reference
=============

Usage
-----

.. code-block:: text

   peter-hook <COMMAND> [OPTIONS]

Commands
--------

install
^^^^^^^

Install hooks for the current repository. Creates managed shell scripts under ``.git/hooks`` that delegate to ``peter-hook``.

Options:

- ``--force``: Backup existing non-managed hooks and install anyway
- ``--worktree-strategy``: Worktree hook installation strategy (shared, per-worktree, detect)

uninstall
^^^^^^^^^

Remove peter-hook managed hooks. Restores backups when present.

Options:

- ``--yes``: Do not prompt for confirmation

run
^^^

Run hooks for a specific git event (e.g., ``pre-commit``, ``pre-push``, ``commit-msg``).

Positional:

- ``event``: Git hook event

Options:

- ``--all-files``: Run on all files instead of only changed files
- ``--dry-run``: Show what would run without executing hooks
- ``git_args``: Additional arguments passed from git

validate
^^^^^^^^

Parse and validate the nearest ``hooks.toml``. Prints discovered hooks and groups.

Options:

- ``--trace-imports``: Show import order, overrides, cycles, and unused imports
- ``--json``: Output diagnostics as JSON (use with ``--trace-imports``)


list
^^^^

List installed hooks in ``.git/hooks`` and show whether they are managed by peter-hook.

run-hook
^^^^^^^^

Run the same hooks that would run during a git operation without performing the git operation.

Positional:

- ``event``: Git hook event to simulate

Options:

- ``--all-files``: Run on all files instead of only changed files
- ``--dry-run``: Show what would run without executing hooks

run-by-name
^^^^^^^^^^^

Run a specific hook by name.

Positional:

- ``hook_name``: Name of the hook to run

Options:

- ``--all-files``: Run on all files instead of only changed files
- ``--dry-run``: Show what would run without executing hooks

list-worktrees
^^^^^^^^^^^^^^

List worktrees and their hook configuration.

config
^^^^^^

Manage global configuration.

Subcommands:

- ``show``: Show current global configuration
- ``init``: Initialize global configuration (``--allow-local`` to enable absolute imports)
- ``validate``: Validate global configuration

version
^^^^^^^

Show version information.

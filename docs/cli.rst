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

- ``--files``: Enable file-based filtering (skip hooks when no matching files changed)
- ``--``: Pass-through additional git arguments (for hooks like ``commit-msg``/``pre-push``)

validate
^^^^^^^^

Parse and validate the nearest ``hooks.toml``. Prints discovered hooks and groups.

list
^^^^

List installed hooks in ``.git/hooks`` and show whether they are managed by peter-hook.

run-hook
^^^^^^^^

Simulate running the same hooks that would run during a git operation without performing the operation.

Positional:

- ``event``: Git hook event to simulate

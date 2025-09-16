Git Integration
===============

Supported Events
----------------

The installer recognizes configuration for these git hooks and installs scripts accordingly:

- pre-commit
- commit-msg
- pre-push
- post-commit
- post-merge
- post-checkout
- pre-rebase
- post-rewrite
- pre-receive
- post-receive
- update
- post-update
- pre-applypatch
- post-applypatch
- applypatch-msg

Installation
------------

.. code-block:: bash

   peter-hook install

With worktree strategy:

.. code-block:: bash

   # Shared hooks across all worktrees (default)
   peter-hook install --worktree-strategy shared

   # Per-worktree hooks
   peter-hook install --worktree-strategy per-worktree

   # Auto-detect existing strategy
   peter-hook install --worktree-strategy detect

Behavior
--------

- Existing non-managed hooks are backed up as ``<hook>.backup`` when ``--force`` is used
- Managed hooks are shell scripts that execute ``peter-hook run <event> ["$@"]``
- Hooks that receive git arguments (e.g., ``commit-msg``) forward them to peter-hook
- Supports both shared and per-worktree hook installation strategies

Uninstall
---------

.. code-block:: bash

   peter-hook uninstall

Restores backups if they exist.

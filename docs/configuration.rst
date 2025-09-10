Configuration
=============

Peter Hook reads configuration from the nearest ``hooks.toml`` file to the current working directory. Child directories override parent configurations: the nearest file wins.

Hook Definition
---------------

.. code-block:: toml

   [hooks.example]
   command = "echo hello"                # string or array form
   # command = ["echo", "hello"]       # preferred for complex commands
   description = "Example hook"
   modifies_repository = false           # true -> runs sequentially
   workdir = "custom/path"              # optional working directory (relative or absolute)
   env = { KEY = "value" }              # environment variables
   files = ["**/*.rs", "Cargo.toml"]  # glob patterns for file targeting
   depends_on = ["format", "setup"]   # hook dependencies
   run_always = false                    # ignore file changes when true

Hook Groups
-----------

Imports (Hook Libraries)
------------------------

You can split reusable hooks/groups into separate TOML files and import them into your project ``hooks.toml``.

.. code-block:: toml

   # hooks.toml
   imports = ["../hooks.lib.toml", ".hooks/common.toml"]

   [groups.pre-commit]
   includes = ["lint", "format", "test"]  # names from imported files

   # Local override wins on same name
   [hooks.lint]
   command = "cargo clippy -- -D warnings"
   modifies_repository = false

Rules:

- Paths are resolved relative to the importing file (absolute paths allowed)
- Imports merge in order; later imports override earlier ones on duplicate names
- Local definitions override imported ones
- Recursive imports are supported with cycle detection

.. code-block:: toml

   [groups.example-group]
   includes = ["hook1", "hook2", "other-group"]
   execution = "parallel"               # sequential | parallel | force-parallel
   description = "Example group"
   # parallel = true                     # deprecated; kept for backward-compat

Execution Strategies
--------------------

- ``sequential``: run hooks one after another, respecting dependencies
- ``parallel``: run read-only hooks together; repository-modifying hooks run after, sequentially
- ``force-parallel``: run all hooks in parallel (unsafe; ignores ``modifies_repository``)

Repository Safety Rules
-----------------------

- Hooks with ``modifies_repository = true`` never run in parallel with other hooks
- Hooks with ``modifies_repository = false`` can run in parallel with each other
- Mixed groups run in phases: safe hooks first (parallel), then modifying hooks (sequential)


File Targeting
==============

Run hooks only when relevant files have changed. File targeting is automatically enabled when hooks define ``files`` patterns:

.. code-block:: bash

   # Hooks with 'files' patterns automatically target changed files
   peter-hook run pre-commit

   # Use --all-files to run on all matching files instead of only changed files
   peter-hook run pre-commit --all-files

Patterns
--------

``files`` uses glob patterns. Examples:

.. code-block:: toml

   files = ["**/*.rs", "Cargo.toml"]                  # Rust
   files = ["**/*.js", "**/*.ts", "package.json"]    # JavaScript/TypeScript
   files = ["**/*.yml", "**/*.yaml"]                   # YAML
   files = ["**/*.md", "docs/**/*"]                    # Documentation

Behavior
--------

- No ``files`` specified → hook always runs
- ``run_always = true`` → hook always runs regardless of changes
- With patterns → hook runs only if any changed file matches

Example
-------

.. code-block:: toml

   [hooks.rust-lint]
   command = "cargo clippy"
   modifies_repository = false
   files = ["**/*.rs", "Cargo.toml"]

   [hooks.critical-security]
   command = "secret-scan"
   modifies_repository = false
   run_always = true

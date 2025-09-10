Dependencies
============

Define execution order between hooks. Dependencies are resolved with topological sorting and executed in phases; hooks without dependencies can run in parallel within a phase.

Example
-------

.. code-block:: toml

   [hooks.format]
   command = "cargo fmt"
   modifies_repository = true

   [hooks.lint]
   command = "cargo clippy"
   modifies_repository = false
   depends_on = ["format"]

   [hooks.test]
   command = "cargo test"
   modifies_repository = false
   depends_on = ["lint"]

   [groups.release]
   includes = ["format", "lint", "test"]
   execution = "sequential"

Rules
-----

- Cycles are detected and reported as errors
- Missing dependency names are reported as errors
- Phases allow safe parallelism for independent hooks

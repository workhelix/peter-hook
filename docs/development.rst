Development
===========

Prerequisites
-------------

- Rust 1.70+
- Git

Common Tasks
------------

.. code-block:: bash

   # Build
   cargo build

   # Run all tests
   cargo test

   # Strict linting
   cargo clippy --all-targets -- -D warnings

   # Format code
   cargo fmt

   # Validate configuration
   cargo run -- validate

   # Run pre-commit hooks locally
   cargo run -- run pre-commit

Building Docs
-------------

These docs are built with Sphinx.

.. code-block:: bash

   # (Optional) create a virtualenv with uv
   uv venv .venv
   source .venv/bin/activate
   uv pip install sphinx furo

   # Build HTML
   cd docs
   uv run sphinx-build -b html . _build/html

   # Alternatively (no venv):
   # uvx sphinx-build -b html docs docs/_build/html

Open ``_build/html/index.html`` in a browser.

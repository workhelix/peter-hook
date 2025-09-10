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

   # (Optional) create a virtualenv
   python3 -m venv .venv
   source .venv/bin/activate
   pip install sphinx furo

   # Build HTML
   cd docs
   sphinx-build -b html . _build/html

Open ``_build/html/index.html`` in a browser.

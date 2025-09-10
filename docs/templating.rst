Templating
==========

Template variables are available in ``command``, ``workdir``, and ``env`` fields. They are resolved at runtime using repository context and environment variables.

Built-in Variables
------------------

- ``${HOOK_DIR}``: Directory containing the ``hooks.toml``
- ``${WORKING_DIR}``: Working directory when the hook runs
- ``${REPO_ROOT}``: Git repository root
- ``${HOOK_DIR_REL}``: ``HOOK_DIR`` relative to repo root
- ``${WORKING_DIR_REL}``: ``WORKING_DIR`` relative to repo root
- ``${PROJECT_NAME}``: Name of directory containing ``hooks.toml``
- ``${HOME_DIR}``: User home directory

Environment Variables
---------------------

All environment variables are available as templates, for example ``${PATH}``, ``${USER}``, ``${CI}``.

Shell-style Expansions
----------------------

A limited shell-style expansion is supported:

- ``${PWD##*/}`` → Basename of the current directory

Examples
--------

.. code-block:: toml

   [hooks.build]
   command = "cargo build --manifest-path=${HOOK_DIR}/Cargo.toml"
   workdir = "${REPO_ROOT}/target"
   env = { CARGO_TARGET_DIR = "${HOOK_DIR}/target", PROJECT_NAME = "${PROJECT_NAME}" }

   [hooks.test-with-context]
   command = ["cargo", "test", "--manifest-path=${HOOK_DIR}/Cargo.toml", "--", "--test-threads=1"]
   description = "Test ${PROJECT_NAME} in ${HOOK_DIR_REL}"

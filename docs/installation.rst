Installation
============

Using GitHub CLI (Recommended)
------------------------------

.. code-block:: bash

   # Create installation directory
   mkdir -p "$HOME/.local/bin"

   # Download and extract latest release directly (v3.0.1)
   gh release download --repo example/peter-hook --pattern '*-apple-darwin.tar.gz' -O - | tar -xz -C "$HOME/.local/bin"

   # Add to PATH if not already present
   echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc  # or ~/.zshrc
   source ~/.bashrc  # or source ~/.zshrc

   # Verify installation
   peter-hook version

Install Script
--------------

.. code-block:: bash

   # Install via curl (internal project)
   curl -fsSL https://raw.githubusercontent.com/example/peter-hook/main/install.sh | bash

Manual Download
---------------

Visit: https://github.com/example/peter-hook/releases (latest: v3.0.1)

From Source
-----------

.. code-block:: bash

   git clone https://github.com/example/peter-hook.git
   cd peter-hook
   cargo build --release
   # Add target/release to your PATH or install the binary appropriately

Prerequisites
-------------

- GitHub CLI (for recommended installation method)
- Rust 1.70+ (for building from source)
- Git


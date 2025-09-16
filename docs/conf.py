# Configuration file for the Sphinx documentation builder.
# See: https://www.sphinx-doc.org/en/master/usage/configuration.html

from datetime import datetime

# -- Project information -----------------------------------------------------

project = "Peter Hook"
author = "Peter Hook Contributors"
copyright = f"{datetime.now().year}, {author}"

# Keep these in sync with Cargo.toml when releasing
version = "3.0.1"
release = version

# -- General configuration ---------------------------------------------------

extensions = [
    "sphinx.ext.autosectionlabel",
    "sphinx.ext.todo",
]

autosectionlabel_prefix_document = True

# Templates and exclusions
templates_path = ["_templates"]
exclude_patterns = [
    "_build",
    "Thumbs.db",
    ".DS_Store",
]

# -- Options for HTML output -------------------------------------------------

# Theme with graceful fallback
try:
    import furo  # type: ignore  # noqa: F401
    html_theme = "furo"
except Exception:
    html_theme = "alabaster"

html_static_path = ["_static"]

# -- Options for todo extension ---------------------------------------------

todo_include_todos = True

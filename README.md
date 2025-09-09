I want to build my own git hooks manager. We use a `monorepository` stucture, and individual paths within that monorepo must be able to have custom hooks. I am currently interested in `pre-commit` and `pre-push`, to start with.


# HOOK DEFINITIONS

Hooks must be defined as shell executables, either as a `bash` script or as an `execve` style list. Hooks **must** be able to be combined into larger groups. For instance, for Python, we want to run `ruff` and `ty` (for type checking), but we also want to group them together as `python.lint`, that will run the `python.ruff` and `python.ty` hooks. I want to have a library of hooks, that individual users can combine and use, while still being able to impose quality bars across the entire repository.


# HIERARCHICAL STRUCTURE

The hooks that will run are those that are defined by the *nearest* definition file; in other words, assuming a definition file called `hooks.toml`:

/projects/hooks.toml /projects/foo/hooks.toml /projects/foo/bar/hooks.toml

A code change in /projects/foo/bar will rn the /projects/foo/bar/hooks.toml; a code change in /projects/foo/baz will run /projects/foo/hooks.toml.


# CONFIGURATION

All hook scripts **MUST** run in the directory of the hook definition file; they must **NOT** run from the git root, unless they are in the root directory. In addition, **all** configuration **MUST** be in `TOML` format.


# IMPLEMENTATION DETAILS

I don't care about the language we're using. I think the correct approach here is to have this code live in a separate repository; then, we should have a quick way to install it, possibly through a `curl | bash` style pipeline. We can open source this eventually.


# CODE QUALITY

**EVERYTHING MUST BE TESTED**. Everything must pass extremely strict linting. We are running on MacOS but will require the code to support Linux, as well.
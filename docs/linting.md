# ADD LINTING MODE TO peter-hook

I want to add a run mode for `peter-hook` that allows us to run the defined hooks against the current directory; in this mode, per-directory hooks run *as if the current directory is the git root*; the file filtering for per-file hooks should return *every matching file* that is not .gitignore'd; and *no git operations should happen*. 

Let's say we have a hook, called `ruff-check`; this hook is defined as so:

```toml
[hooks.ruff-check]
command = ["uvx", "ruff", "check", "--fix"]
description = "Run ruff linter with auto-fix for Python files"
modifies_repository = true
depends_on = ["ruff-format"]
files = ["**/*.py"]
```

When I run `peter-hook lint ruff-check`, I want this to run on *all* python files in the current directory _and subdirectories_. 

Let's also say we have a per-directroy hook, called `unvenv`, defined as such:

```toml
[hooks.unvenv]
command = ["unvenv"]
description = "Prevent pushing Python virtual environments to Git"
modifies_repository = false
execution_type = "per-directory"
files = ["**/*"]
```

If I run `peter-hook lint unvenv`, the unvenv command runs *in the current directory*.

Does this make sense?

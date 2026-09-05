# Jupyter notebook to Python script

Convert a Jupyter notebook to a Python script

Convert a Jupyter notebook (.ipynb) into a plain Python script in the Jupytext / VS Code percent format: each code cell follows a '# %%' marker and each markdown cell becomes a commented '# %% [markdown]' block. Outputs and execution metadata are dropped, so you get just the code and prose. Point it at the .ipynb JSON; set markdown to skip on the step to emit code only.

## Run

```
mog -m ipynb-to-python <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
{
  "cells": [
    {"cell_type": "markdown", "source": ["# Demo notebook\n", "A short intro."]},
    {"cell_type": "code", "execution_count": 1, "outputs": [], "source": ["import math\n", "print(math.pi)"]},
    {"cell_type": "code", "execution_count": 2, "outputs": [], "source": "x = math.sqrt(2)"}
  ],
  "metadata": {"kernelspec": {"name": "python3"}},
  "nbformat": 4,
  "nbformat_minor": 5
}
```

Output:

```
# %% [markdown]
# # Demo notebook
# A short intro.

# %%
import math
print(math.pi)

# %%
x = math.sqrt(2)
```

## Pipeline

- `ipynb_to_python`: Emit code cells after # %% markers; markdown cells as commented blocks

## Tags

`jupyter` `python` `convert` `ml`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

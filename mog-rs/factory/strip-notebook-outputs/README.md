# Strip Jupyter notebook outputs

Clear Jupyter notebook outputs and execution counts

Clear the outputs and execution counts from a Jupyter notebook (.ipynb) so it produces clean git diffs and small commits: sets every cell's outputs to [] and execution_count to null via json_set. The notebook stays a valid .ipynb (unlike ipynb-to-python, which discards structure). Mechanical: it sets these fields on every cell, including markdown cells (a harmless extra field that renderers ignore).

## Run

```
mog -m strip-notebook-outputs <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
{
  "cells": [
    {"cell_type": "markdown", "source": ["# Demo"]},
    {"cell_type": "code", "execution_count": 7, "outputs": [{"output_type": "stream", "name": "stdout", "text": ["3.14159\n"]}], "source": ["import math\n", "print(math.pi)"]}
  ],
  "metadata": {"kernelspec": {"name": "python3"}},
  "nbformat": 4,
  "nbformat_minor": 5
}
```

Output:

```
{
  "cells": [
    {
      "cell_type": "markdown",
      "source": [
        "# Demo"
      ],
      "outputs": [],
      "execution_count": null
    },
    {
      "cell_type": "code",
      "execution_count": null,
      "outputs": [],
      "source": [
        "import math\n",
        "print(math.pi)"
      ]
```

_(... 10 more line(s))_

## Pipeline

- `json_set`: Clear all cell outputs
- `json_set`: Reset execution counts

## Tags

`jupyter` `cleanup` `ml`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

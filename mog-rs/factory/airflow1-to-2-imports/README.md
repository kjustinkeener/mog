# Airflow 1 to 2 import paths

Rewrite Airflow 1.x import paths to 2.x

Rewrite common Airflow 1.x import module paths to their Airflow 2.x locations (the _operator / _hook / _sensor suffix modules were renamed). Covers core operators, hooks, and sensors; contrib and provider-package moves are not handled.

## Run

```
mog -m airflow1-to-2-imports <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
from airflow.operators.bash_operator import BashOperator
from airflow.operators.python_operator import PythonOperator
from airflow.hooks.base_hook import BaseHook
from airflow import DAG
```

Output:

```
from airflow.operators.bash import BashOperator
from airflow.operators.python import PythonOperator
from airflow.hooks.base import BaseHook
from airflow import DAG
```

## Pipeline

- `replace_map`: Rewrite renamed Airflow module paths

## Tags

`codemod` `python` `imports` `migration`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

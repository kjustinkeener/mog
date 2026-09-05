# Reset Markdown task list checkboxes

Reset Markdown task list checkboxes to unchecked

Uncheck every completed task in a Markdown task list, turning - [x] (or - [X], * [x]) back into - [ ]. Handy for reusing a checklist template for a new run. Indentation, the bullet character, and the task text are preserved; unchecked and non-task lines are left alone.

## Run

```
mog -m markdown-uncheck-tasks <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
# Release checklist
- [x] Write the code
- [ ] Review the PR
  - [X] Run the tests
* [x] Tag the release
Some note that is not a task.
```

Output:

```
# Release checklist
- [ ] Write the code
- [ ] Review the PR
  - [ ] Run the tests
* [ ] Tag the release
Some note that is not a task.
```

## Pipeline

- `replace_regex_multiline`: Turn a checked box into an empty one

## Tags

`markdown` `cleanup` `docs`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

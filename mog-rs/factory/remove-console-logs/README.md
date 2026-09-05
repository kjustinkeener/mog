# Remove console.log lines

Delete console.log debug lines from JS or TS

Delete whole lines that are a console.log / console.debug / console.info / console.warn / console.error call, to strip debug output from JavaScript or TypeScript. Only lines whose content is such a call are removed; a multi-line console call, or one buried mid-line, is not handled.

## Run

```
mog -m remove-console-logs <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
const x = compute();
console.log('debug x =', x);
doWork(x);
  console.error('oops');
return x;
```

Output:

```
const x = compute();
doWork(x);
return x;
```

## Pipeline

- `remove_lines_matching`: Drop lines that are a console.* call

## Tags

`javascript` `typescript` `terminal` `observability` `cleanup`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

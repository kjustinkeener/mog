# .properties to JSON

Convert Java .properties to flat JSON

Parse a Java .properties file (key=value or key:value, #/! comments) into a flat JSON object of string values. Dotted keys are kept flat.

## Run

```
mog -m properties-to-json <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
# app config
app.name=MyApp
app.port=8080
debug=true
```

Output:

```
{
  "app.name": "MyApp",
  "app.port": "8080",
  "debug": "true"
}
```

## Steps

- `properties_to_json`: Parse the .properties into a JSON object.

## Tags

`convert` `config` `java` `json`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

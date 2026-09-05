# .properties to YAML

Convert Java .properties to YAML

Convert a Java .properties file (key=value or key:value, #/! comments) to YAML with flat, dotted keys.

## Run

```
mog -m properties-to-yaml <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
# app config
app.name=MyApp
app.port=8080
```

Output:

```
app.name: MyApp
app.port: '8080'
```

## Pipeline

- `properties_to_json`: Parse the .properties into JSON.
- `json_to_yaml`: Render the JSON as YAML.

## Tags

`convert` `config` `java` `yaml`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

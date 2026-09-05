# .env to Docker Compose

Convert .env to a Docker Compose fragment

Turn a .env file into a docker-compose fragment placing the variables under services.<service>.environment. Set the service with -D service=... (default app).

## Run

```
mog -m env-to-compose <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
NAME=App
PORT=8080
```

Output:

```
services:
  app:
    environment:
      NAME: App
      PORT: '8080'
```

## Pipeline

- `env_to_json`: Parse the .env into a JSON object.
- `json_wrap`: Nest it under services.<service>.environment.
- `json_to_yaml`: Render as compose YAML.

## Tags

`convert` `env` `dotenv` `docker` `yaml`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

# Terraform plan to change table

Turn a terraform show -json plan into a CSV change table, riskiest first

Turn the JSON output of `terraform show -json <planfile>` into a CSV change table with columns action,address,type,name,provider: one row per entry in the top-level resource_changes array. INPUT CONTRACT: the JSON document produced by `terraform show -json` (run `terraform plan -out=tf.plan` then `terraform show -json tf.plan > plan.json`). It does NOT read the human-readable `terraform plan` console text, which has no stable format. The change.actions array is collapsed to one word: ["create"] -> create, ["delete"] -> delete, ["update"] -> update, ["no-op"] -> no-op, and either ordering of ["delete","create"] / ["create","delete"] -> replace. Rows are ordered so the destructive changes surface first: replace, delete, update, create, no-op, any other action last, and then by address within each group. provider is the full provider_name string, e.g. registry.terraform.io/hashicorp/aws.

## Run

```
mog -m terraform-plan-to-changes <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
{
  "format_version": "1.2",
  "terraform_version": "1.9.5",
  "resource_changes": [
    {
      "address": "aws_s3_bucket.logs",
      "mode": "managed",
      "type": "aws_s3_bucket",
      "name": "logs",
      "provider_name": "registry.terraform.io/hashicorp/aws",
      "change": {
        "actions": ["create"],
        "before": null,
        "after": { "bucket": "acme-logs" }
      }
    },
    {
      "address": "aws_s3_bucket.assets",
```

_(... 60 more line(s))_

Output:

```
action,address,type,name,provider
replace,aws_iam_role.runner,aws_iam_role,runner,registry.terraform.io/hashicorp/aws
replace,module.network.aws_subnet.private,aws_subnet,private,registry.terraform.io/hashicorp/aws
delete,module.network.aws_nat_gateway.legacy,aws_nat_gateway,legacy,registry.terraform.io/hashicorp/aws
update,aws_s3_bucket.assets,aws_s3_bucket,assets,registry.terraform.io/hashicorp/aws
create,aws_s3_bucket.logs,aws_s3_bucket,logs,registry.terraform.io/hashicorp/aws
no-op,random_pet.suffix,random_pet,suffix,registry.terraform.io/hashicorp/random
```

## Pipeline

- `json_minify`: Collapse the pretty-printed plan document to a single line so the JSON readers below see one record.
- `json_extract`: Emit one compact JSON object per entry of the top-level resource_changes array; everything else in the plan document is dropped.
- `json_extract`: Read the fields of interest out of each resource change into a tab-separated row. change.actions is an array, so its members are collected with a + between them, giving create, delete, update, no-op or delete+create.
- `replace_regex_multiline`: Either ordering of a destroy-and-recreate pair is a replace, and it sorts first as the riskiest change.
- `replace_regex_multiline`: A plain destroy sorts second.
- `replace_regex_multiline`: An in-place change sorts third.
- `replace_regex_multiline`: A new resource sorts fourth.
- `replace_regex_multiline`: An unchanged resource sorts fifth.
- `replace_regex_multiline`: Anything else Terraform may emit (a read, or a future action word) keeps its own label and sorts last, so no row is ever silently dropped.
- `sort_lines`: Sort on the whole line: the leading rank orders the groups riskiest first, and the address (the next field) breaks ties inside a group.
- `replace_regex_multiline`: Drop the sort rank now that the rows are in order.
- `change_delimiter`: Re-join the tab-separated rows as CSV, quoting any field that contains a comma, a quote or a newline (RFC 4180).
- `prepend`: Add the CSV header row.

## Tags

`convert` `terraform` `json` `csv` `infra`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

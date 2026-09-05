# Terraform HCL to OpenTofu

Rewrite Terraform HCL provider host for OpenTofu

Rewrite Terraform HCL config for OpenTofu. OpenTofu is a drop-in fork, so .tf config is nearly identical; this rewrites the one difference that matters in config text: explicit registry.terraform.io provider source addresses become registry.opentofu.org (short-form sources like hashicorp/aws resolve on both and are left alone). NOT handled: the terraform {} block is left as-is (OpenTofu reads it; converting to tofu {} would break Terraform compatibility), Terraform Cloud app.terraform.io references, and CLI/state/lockfile concerns.

## Run

```
mog -m terraform-to-opentofu <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
terraform {
  required_version = ">= 1.5"
  required_providers {
    aws = {
      source  = "registry.terraform.io/hashicorp/aws"
      version = "~> 5.0"
    }
    random = {
      source  = "hashicorp/random"
      version = "~> 3.0"
    }
  }
}

provider "aws" {
  region = "us-east-1"
}
```

Output:

```
terraform {
  required_version = ">= 1.5"
  required_providers {
    aws = {
      source  = "registry.opentofu.org/hashicorp/aws"
      version = "~> 5.0"
    }
    random = {
      source  = "hashicorp/random"
      version = "~> 3.0"
    }
  }
}

provider "aws" {
  region = "us-east-1"
}
```

## Pipeline

- `replace`: registry.terraform.io -> registry.opentofu.org

## Tags

`config` `terraform` `convert` `single-action`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

# Strip mysqldump scaffolding

Strip mysqldump scaffolding, keep table DDL

Remove the non-table-DDL scaffolding a mysqldump emits: version-gated /*! */ comments, DELIMITER blocks (triggers/routines), LOCK TABLES data blocks, view stubs and definitions, and the SET / USE / CREATE DATABASE / DROP TABLE noise. Leaves CREATE TABLE ... ) ENGINE...; blocks. Composable: run first (via run_mog) from a MySQL->target converter. Runs on the raw dump (backticks intact).

## Run

```
mog -m mysql-strip-dump-noise <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
-- MySQL dump 10.13  Distrib 8.0.46, for Linux (x86_64)
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40103 SET TIME_ZONE='+00:00' */;

--
-- Table structure for table `widget`
--

DROP TABLE IF EXISTS `widget`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `widget` (
  `id` int unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(100) NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
/*!40101 SET character_set_client = @saved_cs_client */;

```

_(... 18 more line(s))_

Output:

```

CREATE TABLE `widget` (
  `id` int unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(100) NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
```

## Pipeline

- `eol_lf`: Normalize to LF so the block regexes below are reliable.
- `replace_regex`: Remove DELIMITER blocks whole (triggers and stored routines -> PL/pgSQL, out of scope).
- `replace_regex`: Remove LOCK TABLES ... UNLOCK TABLES data blocks (INSERTs + DISABLE/ENABLE KEYS).
- `replace_regex`: Remove view definitions, both the temporary stub and the final /*!50001 CREATE ... VIEW ... */; (views are out of scope).
- `remove_lines_matching`: Drop remaining single-line noise: version-gated comments, -- comments, bare SET, USE, CREATE DATABASE, DROP TABLE.
- `squeeze_blank_lines`: Collapse the blank lines the removals leave behind.

## Tags

`fragment` `sql` `mysql` `cleanup` `strip`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

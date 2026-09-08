# Chunk text for RAG

Split text into overlapping chunks for RAG

Split a document into overlapping, word-aligned chunks for RAG ingestion or embedding. Each chunk is about `size` characters (snapped to a whitespace boundary so words stay whole), with `overlap` characters carried into the next chunk to preserve context. Chunks are separated by a --- rule; the demo uses small sizes to show the boundaries.

## Run

```
mog -m rag-chunk <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
Retrieval augmented generation splits a document into smaller pieces so an embedding model can index them and a retriever can find the most relevant passages at query time.
```

Output:

```
Retrieval augmented generation splits a document into smaller pieces so an
---
r pieces so an embedding model can index them and a retriever can find the most
---
find the most relevant passages at query time.
```

## Steps

- `chunk_text`: Split into ~80-char chunks with 15 overlap

## Tags

`ai` `fragment` `ml` `text`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

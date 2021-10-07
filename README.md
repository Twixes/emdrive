# Emdrive

Database management system for fast similarity search within metric spaces, written in Rust.

### Data types

| Name | Description | Size on disk | Value bounds |
| --- | --- | --- | -- |
| `UINT8` | unsigned 8-bit integer | 1 byte | ≥ 0 and < 2⁸ |
| `UINT16` | unsigned 16-bit integer | 2 bytes | ≥ 0 and < 2¹⁶ |
| `UINT32` | unsigned 32-bit integer | 4 bytes | ≥ 0 and < 2³² |
| `UINT64` | unsigned 64-bit integer | 8 bytes | ≥ 0 and < 2⁶⁴ |
| `UINT128` | unsigned 128-bit integer | 16 bytes | ≥ 0 and < 2¹²⁸ |
| `TIMESTAMP` | number of milliseconds [since Unix epoch](https://en.wikipedia.org/wiki/Unix_time), saved in a signed 64-bit integer | 8 bytes | ≥ 2⁶³ ms before Unix epoch and < 2⁶³ ms after Unix epoch (around 292 million years in either direction) |
| `STRING(n)` | UTF-8 string | 2+n bytes | ≤ `n` characters, where `n` ≤ 2048 |

Emdrive types are **non-nullable by default**. They can made so simply by wrapping them in `NULLABLE()`. For instance a nullable string of maximum length 20 is `NULLABLE(STRING(20))`.

### Indexes

| Name | Category | Description | Column types | Operators |
| --- | --- | --- | --- | --- |
| `btree` | general | [B+ tree](https://en.wikipedia.org/wiki/B+_tree) | all | `=` (equality) |
| `emtree` | metric | [EM-tree](https://dl.gi.de/bitstream/handle/20.500.12116/648/paper31.pdf) | metric-dependent | `@` (distance) |

### Metrics

| Name | Description | Column types |
| --- | --- | --- |
| `hamming` | [Hamming distance](https://en.wikipedia.org/wiki/Hamming_distance) | `UINT*` |

### Story

Let's imagine you're running an image search engine. As a fan of geese you called it Gaggle.  
Being a search engine operator, you run a bot which crawls pages on the internet.
Every time the bot sees an image, it computes a [perceptual hash](https://en.wikipedia.org/wiki/Perceptual_hashing)
of it and saves it, along with some other metadata, to an Emdrive instance.

We'll be using database `gaggle`. A relevant table schema here may be:

```SQL
CREATE TABLE photos_seen (
    hash UINT8 METRIC KEY USING mtree(hamming),
    url STRING(2048) PRIMARY KEY,
    width UINT32,
    height UINT32,
    seen_at TIMESTAMP
);
```

> Note that column `hash` is marked with `METRIC KEY USING hamming`!  
While a primary key is B+ tree-based and allows for quick general lookups of rows, it's useless for distance queries.
An EM-tree-based metric key does the job very well though. In this case, as we're comparing perceptual hashes in integer form, Hamming distance
is the most relevant metric.

Oh, your bot has just seen a new image! Let's register it:

```SQL
INSERT INTO photos_seen (hash, url, width, height, seen_at)
VALUES (0b11001111, 'https://twixes.com/a.png', 1280, 820, '2077-01-01T21:37');
```

Now, look, a user just uploaded their image to see similar occurences of it from the internet. The search engine
calculated that image's hash to be `0b00001011` (binary representation of decimal `11`).  
Let's check that against Emdrive. We'll be using the `@` distance operator, which always returns a number
and is exclusively supported for `METRIC KEY` columns.

```SQL
SELECT url, hash @ 0b00001011 AS distance FROM photos_seen WHERE distance < 4;
```

It's a match! The image we saved previously has a similar hash, and we can now show it in search results.

| `url`                        | `distance` |
| ---------------------------- | ---------- |
| `"https://twixes.com/a.png"` | `3`        |

### Data storage

```bash
$EMDRIVE_DATA_DIRECTORY # /var/lib/emdrive/data by default
└── databases/
   └── gaggle/ # database
      └── tables/
         └── photos_seen/ # table
            └── data # core table data
            └── indexes/ # table indexes, used for quicker row lookup
               └── hash-emtree-hamming # bplustree index on column url
            └── meta # table metadata
```

#### Data structure

Every table has a `data` file containing all its, well, data. Such `data` files are made up of **pages**.

#### Page structure

Every page is 8 KiB in size.

There are 3 page types:
- meta - The initial page, contains directions for the whole `data` file.
    - page type (1 byte, `u8`) - `0x00`.
    - schema version (2 bytes, `u16`) - `0x0000` currently.
    - B+ tree root page index (4 bytes, `u32`)
- B+ tree node
    - page type (1 byte, `u8`) - `0x30`.
    - row count (8 bytes, `u64`) - Number of rows contained by all child leaves of the node. A `COUNT` and `OFFSET` optimization.
    - fan-out (2 bytes, `u16`) - Number of keys in this node.
    - child page indexes ((fan-out + 1) * 4 bytes, `u32`) - Child pointers.
    - keys (fan-out of them) - Key values.
- B+ tree leaf
    - page type (1 byte, `u8`) - `0x31`.
    - next leaf page index (4 bytes, `u32`)
    - row count (2 bytes, `u16`) - Number of rows in this leaf.
    - rows (row count of them) – See [row structure](#row-structure).

> Data stored by Emdrive on disk is big-endian, meaning that less significant bytes have higher addresses than more significant bytes (similar to how humans write numbers down).

#### Row structure

Rows contain values in the same order as their columns in the table definition, except variable-length values come after all fixed-length ones.
Variable-length values are also prefixed with 2 length bytes (`u16`).
Nullable values are prefixed with 1 marker byte (`u8`), which is `0x00` if the value is null, otherwise `0x01`.

### Launch configuration

The following launch configuration settings are available for Emdrive instances.
They are applied on instance launch from environment variables in the format `EMDRIVE_${SETTING_NAME_UPPERCASE}`
(i.e. setting `data_directory` is set with variable `EMDRIVE_DATA_DIRECTORY`).
If a setting's environment variable is not set, its default value will be used.

| Name | Type | Default value | Description |
| --- | --- | --- | --- |
| `data_directory` | `STRING` | `"/var/lib/emdrive/data"` | Location of all data, including system tables |
| `http_listen_host` | `STRING` | `"127.0.0.1"` | Host on which the HTTP server will listen |
| `http_listen_port` | `UINT16` | `8824` | Port on which the HTTP server will listen |

### Search

### SQL

### HTTP interface

## Benchmarks

| Postgres | MySQL | ClickHouse | ⚡️ Emdrive |
| --- | --- | --- | --- |

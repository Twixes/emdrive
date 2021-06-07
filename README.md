# Metrobaza

Database for fast similarity search within metric spaces, written in Rust.

### Data types

| Name | Description | Size on disk | Value bounds |
| --- | --- | --- | -- |
| `UINT8` | unsigned 8-bit integer | 1 byte | ≥ 0 and < 2⁸ |
| `UINT16` | unsigned 16-bit integer | 2 bytes | ≥ 0 and < 2¹⁶ |
| `UINT32` | unsigned 32-bit integer | 4 bytes | ≥ 0 and < 2³² |
| `UINT64` | unsigned 64-bit integer | 8 bytes | ≥ 0 and < 2⁶⁴ |
| `UINT128` | unsigned 128-bit integer | 16 bytes | ≥ 0 and < 2¹²⁸ |
| `TIMESTAMP` | number of milliseconds [since Unix epoch](https://en.wikipedia.org/wiki/Unix_time), saved in a signed 64-bit integer | 8 bytes | ≥ 2⁶³ ms before Unix epoch and < 2⁶³ ms after Unix epoch (around 292 million years in either direction) |
| `VARCHAR(n)` | UTF-8 string | 2+n bytes | ≤ n characters, where n < 2¹⁶ |

## Design

Let's say you're running an image search engine. As a fan of geese you called it Gaggle.  
Being a search engine operator, you run a bot which crawls pages on the internet.
Every time the bot sees an image, it computes a [perceptual hash](https://en.wikipedia.org/wiki/Perceptual_hashing)
of it and saves it, along with some other metadata, to a Metrobaza instance.

We'll be using database `gaggle`. Here's what a relevant schema may look like:

```SQL
CREATE TABLE photos_seen (
    hash UINT8 METRIC KEY USING hamming,
    url VARCHAR(2048) PRIMARY KEY,
    width UINT32,
    height UINT32,
    seen_at TIMESTAMP
);
```

> Note that column `hash` is marked with `METRIC KEY USING Hamming`!  
This is somewhat similar to what in many databases would be `PRIMARY KEY`.
The difference is that Metrobaza is explicitly oriented around the mathematical concept of
a [metric space](https://en.wikipedia.org/wiki/Metric_space). That is so what makes is to suitable for similarity search
AND somewhat different in its workings.  
A metric key must specify a metric. Since this is a hash, `HammingDistance` is the appropriate metric.  
If the column contained text, we'd likely go for `LevenshteinDistance`.  

Your bot has just seen a new image!  
Let's save it.

```SQL
INSERT INTO photos_seen (hash, url, width, height, seen_at)
VALUES (0b11001111, "https://twixes.com/a.png", 1280, 820, Now());
```

Now, look, a user just uploaded their image to see similar occurences of it from the internet. The search engine
calculated that image's hash to be `0b00001011` (binary representation of decimal `11`).  
Let's check that against Metrobaza. We'll be using the `@` distance operator, which always returns a number
and is radically optimized for `METRIC KEY` columns.

```SQL
SELECT url, hash @ 0b00001011 AS distance FROM photos_seen WHERE distance < 4;
```

It's a match! The image we saved previously has a somewhat similar hash, and we can now show it in search results.

| `url`                        | `distance` |
| ---------------------------- | ---------- |
| `"https://twixes.com/a.png"` | `3`        |

### Data storage

```bash
$METRO_DATA_DIRECTORY # /var/lib/metrobaza/data by default
└── system/
└── databases/
   └── gaggle/ # database
      └── tables/
         └── photos_seen/ # table
            └── data/ # table rows
               └── 0 # segment 0 of row data
            └── indexes/ # table indexes, used for quicker row lookup
               └── url#bplustree # bplustree index on column url
            └── metrics/ # table metrics, used for distance-based search
               └── hash#hamming # hamming metric on column hash
            └── meta # table metadata
```

#### Row data structure

> General note: Metrobaza is big-endian, regardless of target architecture.

Each row in a table has the same size, calculated based on table columns and then rounded up to the nearest power of 2
OR the nearest multiple of 4096 – whichever lowest.

For instance our exemplary database has columns:

| Name | Type | Size |
| --- | --- | --- |
| hash | `UINT8` | 1 content byte |
| url | `VARCHAR(2048)` | 2 offset bytes + 2048 content bytes |
| width | `UINT32` | 4 content bytes |
| height | `UINT32` | 4 content bytes |
| seen_at | `TIMESTAMP` | 8 content bytes |

This sums up to 2067 bytes. Rounding up, each row is 4096 bytes (the extra 2029 are reserved as zeroes).

##### Why round up?

This is to reduce the number of reads and writes across disk blocks, whose size is a power of 2 – commonly 4096 bytes.

#### Nullability

Metrobaza types are **non-nullable by default**. They can made so by simply wrapping them in `NULLABLE()` when defining
the table. For instance a nullable string of maximum length 20 is `NULLABLE(VARCHAR(20))`.
Values of nullable columns are prefixed with a marker byte. If the value _is_ null, that byte is 0, and so are all the other bytes of that value. Otherwise that byte is 1.

### Metrics

| Name | Description | Column types |
| --- | --- | --- |
| `hamming` | [Hamming distance](https://en.wikipedia.org/wiki/Hamming_distance) | `UINT8`, `UINT16`, `UINT32`, `UINT64`, `UINT128` |
<!--| `levenshtein` | [Levenshtein distance](https://en.wikipedia.org/wiki/Levenshtein_distance) | `VARCHAR` |-->

### Launch configuration

The following launch configuration settings are available for Metrobaza instances.
They are applied on instance launch from environment variables in the format `METRO_${SETTING_NAME_UPPERCASE}`
(i.e. setting `data_directory` is set with variable `METRO_DATA_DIRECTORY`).
If a setting's environment variable is not set, its default value will be used.

| Name | Type | Default value | Description |
| --- | --- | --- | --- |
| `data_directory` | `String` | `"/var/lib/metrobaza/data"` | Location of all data, including system tables |
| `http_listen_host` | `String` | `"127.0.0.1"` | Host on which the HTTP server will listen |
| `http_listen_port` | `U16` | `8824` | Port on which the HTTP server will listen |

### Search

### SQL

### HTTP interface

## Benchmarks

Postgres vs MySQL vs. ClickHouse vs. Metrobaza

## Etymology

metryka (metric) + baza danych (database) = Metrobaza

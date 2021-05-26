# Metrobaza

Database for fast similarity search within metric spaces, written in Rust.

## Design

Let's say you're running an image search engine. As a fan of geese you called it Gaggle.  
Being a search engine operator, you run a bot which crawls pages on the internet.
Every time the bot sees an image, it computes a [perceptual hash](https://en.wikipedia.org/wiki/Perceptual_hashing)
of it and saves it, along with some other metadata, to a Metrobaza instance.

We'll be using database `gaggle`. Here's what a relevant schema may look like:

```SQL
CREATE TABLE photos_seen (
    hash U8 METRIC KEY USING HammingDistance,
    url String,
    width U32,
    height U32,
    seen_at Timestamp
);
```

> Note that column `hash` is marked with `METRIC KEY USING HammingDistance`!  
This is very similar to what in many databases would be `PRIMARY KEY`.
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
Let's check that against Metrobaza. We'll be using the `@` distance operator, which only works on `METRIC KEY` columns
and always returns a number.

```SQL
SELECT url, hash @ 0b00001011 AS distance FROM photos_seen WHERE distance < 4;
```

It's a match! The image we saved previously has a somewhat similar hash, and we can now show it in search results.

| `url`                         | `distance` |
| ----------------------------- | ---------- |
| `"https://twixes.com/a.png"` | `3`        |

### Data types

| Name | Description | Bounds |
| --- | --- | --- |
| `U8` | unsigned 8-bit integer | at least 0, up to 2⁸-1 |
| `U16` | unsigned 16-bit integer | at least 0, up to 2¹⁶-1 |
| `U32` | unsigned 32-bit integer | at least 0, up to 2³²-1 |
| `U64` | unsigned 64-bit integer | at least 0, up to 2⁶⁴-1 |
| `U128` | unsigned 128-bit integer | at least 0, up to 2¹²⁸-1 |
| `Timestamp` | number of milliseconds [since Unix epoch](https://en.wikipedia.org/wiki/Unix_time), saved in a signed 64-bit integer | at least 2⁶³ ms before Unix epoch, up to 2⁶³-1 seconds after Unix epoch (around 292e6 years in either direction) |
| `String` | UTF-8 string | no more than 2¹⁶-1 characters |

### Metrics

| Name | Description | Column types |
| --- | --- | --- |
| `HammingDistance` | [Hamming distance](https://en.wikipedia.org/wiki/Hamming_distance) | `U8`, `U16`, `U32`, `U64`, `U128` |
<!--| `LevenshteinDistance` | [Levenshtein distance](https://en.wikipedia.org/wiki/Levenshtein_distance) | `String` |-->

### Data storage

```
/var/lib/metrobaza/
└── data
   └── gaggle
      └── photos_seen
         ├── data
         └── index
```

### Search

### SQL

### HTTP interface

## Benchmarks

Postgres vs MySQL vs. ClickHouse vs. Metrobaza

## Etymology

metryka (metric) + baza danych (database) = Metrobaza

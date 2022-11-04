# Random People Generator

This package is a simple Rust-based command-line tool to create fake people
records, using first, middle, and last names taken at random from United
States Census Bureau data that's captured in local files. By default, it
splits the generated names so that half are female and half are male,
but that can be changed via command line options.

(Yes, I know there are more than two genders. I support that distinction.
The Census Bureau data files I'm using are from 2010, and they only supported
two genders. For now, this program is consistent with that restriction, though
I'm considering ways to expand it to generate data that's more reflective of
gender reality.)

This is a Rust version of the original Scala
[peoplegen](https://github.com/bmc/peoplegen) tool I built awhile ago. I
made this Rust version as part of my "learning Rust" journey.

The tool can generate CSV or JSON output.

As is probably obvious, I use this program to generate test data.

----

**WARNING:** I built this tool for myself, as a concrete programming exercise
while learning Rust. (I'm _still_ learning Rust.) You're welcome to use it,
read it, comment constructively on any non-idiomatic Rust you find, etc.
However, do _not_ expect me to maintain this tool rigorously. It's a playground
for me, as well as something I use occasionally. That's all.

----

## Installation

Clone this repo in the usual way. Ensure that you have a suitable, up-to-date
Rust environment installed, along with some version of `make`. Then, simply
type:

```
$ make install
```

The compiled binary will be copied to `$HOME/bin`, and the supporting text
files will be copied to `$HOME/etc`. The build process will also echo some
environment variable settings to standard output; setting those environment
variables will save you having to enter three command line options every time
you run `peoplegen`.

If you'd prefer to install every under, say, `/usr/local` (e.g.,
`/usr/local/bin`, `/usr/local/etc`), simply change `BASE_DIR` at the top
of the `Makefile`.

## Usage

At any time, you can run `peoplegen --help` for a usage summary.

## Output Formats

The `-o` option specifies the desired output format. There are three
possible values:

**`csv`**

Generate a CSV file, with a header. The output file must
end with `.csv`.

**`json`**

Generate a "normal" JSON file. The output is of the following form (though it
is _not_ pretty-printed):

```json
{"people" [
  { "first_name": "Moe", ... },
  { "first_name": "Larry", ... },
  { "first_name": "Curly", ... },
  ...
]}
```

* `jsonl`

Creates a [JSON Lines](https://jsonlines.org/) file from a vector of randomly
generated `Person` objects. JSON Lines is a line-by-line JSON format, where
each object occupies its own text line, and there's no enclosing object or
array. For instance:

```json
{ "first_name": "Moe", ... },
{ "first_name": "Larry", ... },
{ "first_name": "Curly", ... },
...
```

JSON files of this form are well-suited for ingesting into distributed
systems such as Apache Spark, for processing with line-based Unix tools,
etc.

## License

See the `LICENSE` file in the source distribution. (Basically, I don't
care what you do with this code. Just don't bug me about it.)

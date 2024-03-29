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
made this Rust version as part of my "learning Rust" journey. One advantage
this version has over the Scala version: The Rust version is considerably
faster, since it compiles down to a native executable.

The tool can generate CSV or JSON output.

As is probably obvious, I use this program to generate test data.

----

**WARNING:** I built this tool for myself, as a concrete programming exercise
while learning Rust. (I'm *still* learning Rust.) You're welcome to use it,
read it, comment constructively on any non-idiomatic Rust you find, etc.
However, do *not* expect me to maintain this tool rigorously. It's a playground
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

The output file extension dictates the output format. The following
extensions are supported:

**`.csv`**

Generate a CSV file, with a header. The output file must
end with `.csv`.

**`.json`**

Generate a "normal" JSON file. The output file must end with `.json`.
The JSON is of the following form (though it is *not* pretty-printed):

```json
{"people": [
  {"first_name":"Cleveland","middle_name":"Darren","last_name":"McQuaid","gender":"M","birth_date":"1993-01-14","ssn":"934-79-3074"},
  {"first_name":"Percy","middle_name":"Jasper","last_name":"Drohane","gender":"M","birth_date":"1951-01-27","ssn":"963-73-1208"},
  {"first_name":"Aurora","middle_name":"Sanora","last_name":"Crookshank","gender":"F","birth_date":"1997-09-14","ssn":"967-41-1818"},
  ...
]}
```

**`.jsonl`**

Creates a [JSON Lines](https://jsonlines.org/) file from a vector of randomly
generated `Person` objects. The output file must end with `.json`.

JSON Lines is a line-by-line JSON format, where each object occupies its own
text line, and there's no enclosing object or array. For instance:

```json
{"first_name":"Cleveland","middle_name":"Darren","last_name":"McQuaid","gender":"M","birth_date":"1993-01-14","ssn":"934-79-3074"}
{"first_name":"Percy","middle_name":"Jasper","last_name":"Drohane","gender":"M","birth_date":"1951-01-27","ssn":"963-73-1208"}
{"first_name":"Aurora","middle_name":"Sanora","last_name":"Crookshank","gender":"F","birth_date":"1997-09-14","ssn":"967-41-1818"}
...
```

JSON files of this form are well-suited for ingesting into distributed
systems such as Apache Spark, for processing with line-based Unix tools,
etc.

## Salaries

`peoplegen` can optionally generate a salary for each person. It generates
salaries as a normal distribution of integers, around a mean of $58,260
(the United States mean salary across all occupations, for 2021, according to
the [Bureau of Labor Statistics](https://www.bls.gov/oes/current/oes_nat.htm)).
It uses a sigma (i.e., a "spread", or the standard deviation) of 5,000 by
default.

You can change both of those numbers using `--salary-mean` and
`--salary-sigma`, respectively. For instance, if you want to use the mean
salary for computer programmers in 2021, specify `--salary-mean 120990`.

**Warning**: changing either or both values *can* result in negative salaries,
which will cause `peoplegen` to abort.

## About those Social Security numbers

`peoplegen` will optionally generate United States Social Security numbers for
each person. The generated Social Security numbers are deliberately invalid,
generated as described here: <https://stackoverflow.com/a/2313726/53495>

Specifically, this program generates Social Security numbers with prefixes
in the range 900-999, plus the prefix 666. This works out to a total of
99,980,001 possible Social Security numbers. **If you generate more than
99,980,001 people, some Social Security numbers *will* be reused.**

## License

See the `LICENSE` file in the source distribution. (Basically, I don't
care what you do with this code. Just don't bug me about it.)

<div align="center">

# history-grep(hg)

history-grep is a command line tool to search history files with search terms.

The default behavior is to look for the default shell history file and parse it with as many space seperated terms as you like.

``` sh
$ hg docker exec

: 1688074173:0;docker exec -it airflow bash
: 1688155526:0;docker exec -it db1c344458cf pyspark
: 1687987518:0;docker exec -it tabulario/iceberg-rest bash
: 1685566287:0;docker exec -d airflow airflow scheduler
```

</div>

[Flags](#flags) •
[Installation](#installation) •
[Examples](#examples) •
[Todo](#todo) •

## Flags

``` sh
Usage: history-grep [OPTIONS] [PATTERN]...

Arguments:
  [PATTERN]...  Sequence of search terms used to select matching lines

Options:
      --history  Select a history file to search from home folder
  -f, --file     Select a complete file path to search
  -h, --help     Print help
```

## Installation

1. Clone the repo and `cd` into the root.

2. Install binary as `hg`;

`cargo install --path .`

## Examples


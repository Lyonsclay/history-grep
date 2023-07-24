<div align="center">

# history-grep(hg)

history-grep is a command line tool to search history files with search terms.

</div>

The default behavior is to look for the default shell history file and parse it with as many space seperated terms as you like.

``` sh
$ hg docker exec

: 1688074173:0;docker exec -it airflow bash
: 1688155526:0;docker exec -it db1c344458cf pyspark
: 1687987518:0;docker exec -it tabulario/iceberg-rest bash
: 1685566287:0;docker exec -d airflow airflow scheduler
```


[Flags](#flags) •
[Installation](#installation) •
[Examples](#examples) •
[Todo](#todo) 

## Flags

``` sh
Usage: hg [OPTIONS] [SEARCH_TERMS]...

Arguments:
  [SEARCH_TERMS]...  Sequence of search terms used to select matching lines

Options:
      --history  Select a history file to search from home folder
  -f, --file     Select a complete file path to search from
  -d, --dedupe   Dedupe output lines
  -h, --help     Print help
```

## Installation

1. Clone the repo and `cd` into the root.

2. Install binary as `hg`;

`cargo install --path .`

If you are installing an update.

`cargo install --path . --force`

## Examples

1. Expressions with a command line flag(leading dash).

``` sh
$ hg -- rm -rf

Searching for ["rm", "-rf"] in "/Users/clay/.zsh_history"
rm -rf venv
rm -rf ~/.git
rm -rf data/may_examples
```

Will work also; `hg rm -- -rf`

``` sh
hg -- -it -n

Searching for ["-it", "-n"] in "/Users/clay/.zsh_history"
docker run -it --network dbt minio/mc ls local
kubectl exec -it airflow-scheduler-0 -n airflow -- sh
```

2. Expressions with special characters.

Use an escape(back slash) before the character.
``` sh
$ hg -- rm \*

Searching for - ["-rf", "rm", "*"] - in "/Users/clay/.zsh_history"
rm -rf ~/.pyenv/shims/jupyter-*
rm -rf /Users/clay/dev/airflow/dags/file_transfers/*
rm source/events/**/*.log
```

## Shells

zsh
* Default log format: `: <beginning time>:<elapsed seconds>;<command>`

bash
* Default log format: `<command>`
* Can take $HISTFILEFORMAT env var to alter format.

## Todo

* Add search for log files.

* Add shell enum structs encapsulating attributes and format patterns.

* Output range of rows. 

* Capture lines that are not utf-8 in `History` field `not_utf8`.

* `read_lines` from relative file path.

* Use `--file` flag non-interactively.


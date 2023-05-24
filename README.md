# mdb - markdownbrain

## Installation

```bash
cargo install --path .
```

## Usage

Create a config in `~/.config/mdb/config.toml`

```toml
[config]
data = "~/.mdb/db.toml"

[[templates]]
id = "default"
name.exec = { run = "bash", args = ["-c", "date -u +%Y-%m-%d|tr -d '\n'"] }

[[templates]]
id = "readme"
name.text = "README"
content = """
# $NAME

## Installation

```bash
```

## Usage

```bash
```

## Contributing


## License

[MIT](https://choosealicense.com/licenses/mit/)
"""
```

Then run 

```bash
# run default and create a file named YYYY-MM-DD.md in your $PWD
mdb

# open or create a README.md in your $PWD
mdb readme

# create/overwrite a new TESTS.md based on the `readme` template in you $PWD
mdb new -t readme TESTS

# add an existing file to the mdb
mdb add CONTRIBUTORS.md

# list files opened with mdb
mdb list

# search for content
mdb list | xargs rg $(read)
```

## Contributing

Issues (ideas, bugs, whatever) and PRs are very much welcome!


## License

[MIT](https://choosealicense.com/licenses/mit/)

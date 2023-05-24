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

## Usage

## Contributing

## License

[MIT](https://choosealicense.com/licenses/mit/)
"""
```

Create `~/.config/mdb/default.md` for the default template to run OK.

```bash
mkdir -p ~/.config/mdb
printf '# $NAME\n\n' > ~/.config/mdb/default.md
```

Then run 

```bash
# run default and create a file named YYYY-MM-DD.md in your $PWD
mdb

# open or create a README.md in your $PWD
mdb readme

# create/overwrite a new TEMP_FILE_MDB.md based on the `readme` template in you $PWD
mdb new -t readme TEMP_FILE_MDB

# add an existing file to the mdb
# this will add the default created by `mdb` above
mdb add $(date -u +%Y-%m-%d|tr -d '\n').md

# list files opened with mdb
mdb list

# search for content
mdb list | xargs rg $(read)

# example bulk add files to db
find ~ -name 'README.md' -path '*/github.com/*' -not -path '*/node_modules/*' | xargs -I {} mdb add -- "{}"

# after a while, the db might have files that are no longer present so clean it up
mdb clean
```

## Contributing

Issues (ideas, bugs, whatever) and PRs are very much welcome!


## License

[MIT](https://choosealicense.com/licenses/mit/)

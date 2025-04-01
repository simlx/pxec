# pxec

`pxec` / `pxc` is a tool that allows you to execute **scripts** through aliases.

## Configuration

You can configure `pxec` by editing the configuration file located at `/config/config`. Currently, the only configurable option is the editor.

## Building

To build the project in release mode, use the following command:

```bash
cargo build --release
```

# Supported Platforms

`Linux`

# How to use

```
lsc                    -> List all categories.
(ls | list)            -> List all commands.
(ls | list) <name>     -> List all commands in category <name>.
edit <name>            -> Edit the command <name>.
add <name>             -> Add a new command with the name <name>.
print <name>           -> Print the content of the command <name>.
ext | external         -> Export the command <name>.
rm | remove            -> Remove the command <name>.
```

# Directory structure

```
.pxc/
├── cmd/                # Contains script commands (identified by unique hashes).
│   ├── 0EE20629
│   ├── 103A40A7
│   ├── F7265AAD
│   ├── F8D925B0
│   └── FFED6378
├── config/             # Configuration files.
│   └── config
└── map/                # Mapping information.
```

# License

This project is licensed under the GPLv3 License.

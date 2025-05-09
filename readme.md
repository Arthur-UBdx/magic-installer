# Minecraft Magic Installer

This is a simple program that will install all the mods you need to play on a private server, you can change the modpack and mod loader in the `config` file.

## Config

The config.txt file is a simple text file that contains the following:
(default config)

```py
###############
# CONFIG FILE #
###############
# - the '#' must be at the start of a line and denotes comments, they are not taken into account.
# - avoid blank lines or use a single '#'
###############
#
#
# the url to download the mods, config, etc...
modpack_url=
# the url to download the modloader installer (fabic, neoforge, etc...)
modloader_url=
#
# the name of the executable of the modloader installer (.jar OR .exe)
modloader_execname=fabric-installer.jar
#
# the folders to overwrite with the content of the modpack (separated by ,) other folders will be merged with the content of the modpack
# for example, you'd want to overwrite the mods folder to not merge with mods that were there before, on the other hand, you wouldn't like
# to delete all your texturepacks/shaderpacks and replace them with the one included with the modpack
folders_to_overwrite=mods,config,defaultconfig,kubejs,scripts,panoramas
```

## Overriding the default minecraft folder

On Windows, the default minecraft folder is located in `C:\Users\<username>\AppData\Roaming\.minecraft` / `%appdata%\.minecraft`
On Linux, the default minecraft folder is located in `~/.minecraft`

You can specify a different folder by setting the `MINECRAFT_DIR` environment variable to the path of the folder you want to use.
You can use environnement variables, for example: `$home/my_folder/.minecraft` is valid.

## Running tests

The unit tests are located in each corresponding module, they are run with `cargo test`,
for running certains test, like `test_download` in `files.rs` module, you need to have `nodejs` installed.
You can set `KEEP_FILES` to `true` to prevent the cleaning of processeded files by the tests.

## changelog

### 1.0.0

- Initial release

### 2.1.0

- Added support for dropbox zip files

### 2.2.0

- Added UNIX support.
- Added support for modifiable config file.

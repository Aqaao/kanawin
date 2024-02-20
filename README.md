
## Kanawin

This is a plugin for [kanata](https://github.com/jtroo/kanata), automatically change layer by detecting current process.

- No dependencies required.
- Windows only.

## Configuration

Layer check in order from top to bottom, until match successful.

Layer change only occur in the layers defined in config file. If current kanata's layer is not exist in config file, it won't be changed.

```yaml
# "exe" can be full path or part of full path.
# Use "*" match all processes(This is not wildcard nor regular), please put it on last.

# "target_layer" must be exist in kanata config,
# kanawin can't check its correctness and can't handle error!!!

- exe: "C:\\path\\ProjectZomboid\\ProjectZomboid64.exe"
  target_layer: "game_layer"
- exe: "firefox.exe"
  target_layer: "browser_layer"
- exe: "*"
  target_layer: "default_layer"
```

## Run

First run kanata with TCP sever, pass `-p` flag and port.

```shell
kanata.exe -p 1234
```

Then run kanawin with administrator.
This will look for the configuration file `kanawin.yaml` in executable file directory.

```shell
kanawin.exe -p 1234
```

or pass config file path.

```shell
kanawin.exe -p 1234 -c D:/path/kanawin.yaml
```

set environment variable ```RUST_LOG="debug"``` get more console information.

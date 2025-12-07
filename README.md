# clippers

A minimal CLI-based clipboard manager, for Linux (Wayland) and MacOS. Inspired by [cliphist](https://github.com/sentriz/cliphist).

- Recall history with pickers like [rofi](https://github.com/davatorium/rofi) on Linux and [choose](https://github.com/chipsenkbeil/choose) on macOS
- Supports various MIME types, including text and images

## Usage

### Watch for clipboard changes
```sh
clippers watch
```

### Select from history

**Linux: With rofi (via dmenu mode)**
```sh
clippers list | sed 's/:::/\x0/g' | rofi -sep '\0' -dmenu | clippers pick
```

**macOS: With choose**
```sh
clippers list | choose -x ::: | clippers pick
```

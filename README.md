# filecat 🐱

A lightning-fast file concatenation tool written in Rust that allows you to combine multiple files with powerful filtering capabilities.

## Features ✨

- 🚀 Fast and memory-efficient file processing
- 📁 Recursive directory scanning
- 🔍 Filter files by extensions
- ⛔ Exclude files using glob patterns
- 🎯 Optional inclusion of files without extensions
- 🌈 Colorized output for better readability
- 📊 Progress bar for large operations
- ⚡ Asynchronous file reading

## Installation 📦

```bash
cargo install filecat
```

## Usage 🛠️

Basic usage:
```bash
filecat /path/to/directory
```

### Options

```bash
USAGE:
    filecat [OPTIONS] <PATH>

ARGS:
    <PATH>    Directory to scan for files

OPTIONS:
    -e, --ext <EXTENSIONS>     File extensions to include (comma-separated)
                              Example: -e js,ts,jsx
    
    -x, --exclude <PATTERNS>   Patterns to exclude (comma-separated)
                              Example: -x "node_modules,**/test/**"
    
    -n, --include-no-ext      Include files without extensions
    
    -h, --help                Print help information
    
    -V, --version             Print version information
```

### Examples 📝

1. Concatenate all JavaScript files:
```bash
filecat -e js /path/to/project
```

2. Concatenate JavaScript and TypeScript files, excluding tests:
```bash
filecat -e js,ts -x "**/test/**,**/*.test.*" /path/to/project
```

3. Include files without extensions:
```bash
filecat -e js,ts -n /path/to/project
```

## Contributing 🤝

Contributions are welcome! Please feel free to submit a Pull Request.

## License 📄

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
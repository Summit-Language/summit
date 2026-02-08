# Summit Programming Language

A simple, statically-typed systems language that transpiles to C

    Warning: Summit is in early development. Expect bugs, breaking changes, and missing features.

## About Summit

Summit is a modern systems programming language with C/C++ inspired syntax, designed for simplicity and performance. It transpiles to readable C code, using existing C compilers for optimization and cross-platform support.

### Key Features:

- Static typing with type inference
- "C++"-like syntax but with modern conveniences, and less dense syntax
- Zero-cost abstractions
- Direct C interoperability

## Platform Support

- **Linux**: Supported (any mainline distribution)
- **Windows**: Not officially supported yet, but planned for future releases
- **macOS**: Untested

## Building from Source

### Linux

#### Quick Install
```bash
git clone https://github.com/Summit-Language/summit.git
cd summit
./install.sh
```

This will:
- Build the Summit compiler in release mode
- Install the `summit` binary to `/usr/local/bin`
- Build and install the standard library
- Set up environment variables

After installation, restart your shell or source your config:
```bash
source ~/.bashrc # or ~/.zshrc
```

#### Manual Build

Otherwise, if you prefer to build manually:
```bash
# Clone and CD into the repository
git clone https://github.com/Summit-Language/summit.git
cd summit

# Build the compiler
cargo build --release

# Build and install the standard library
cd stdlib
make install-global
cd ..

# Optionally, copy the binary to your PATH
sudo install -m 755 target/release/summit /usr/local/bin/summit
```

#### Uninstall
```bash
cd summit
./uninstall.sh
```

This removes the compiler binary, standard library, and environment variables.

### Requirements

- Rust toolchain 1.93.0+
- Cargo 1.93.0+
- GCC 15.2.1+ or Clang
- GNU Make 4.4.1+

**Check your versions:**
```bash
rustc --version && cargo --version && gcc --version && make --version
```

## Usage
```bash
summit build your_program.sm
```
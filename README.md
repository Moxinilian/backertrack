# backertrack

[![MIT][s1]][l1] ![Lines of Code][s3] [![Join us on Discord][s2]][l2]

[l1]: LICENSE
[s1]: https://img.shields.io/badge/license-MIT-blue.svg
[s2]: https://img.shields.io/discord/425678876929163284.svg?logo=discord
[l2]: https://discord.gg/GnP5Whs
[s3]: https://tokei.rs/b1/github/Moxinilian/backertrack?category=code

This utility helps managing the treasury paperwork of the Amethyst Foundation.  
Currently, its only purpose is to keep a ledger of all transactions and automate the processing of donations.  
It can also output documents required by tax authorities.

**NOTE: The command line graphical interface only works on Linux and macOS for now.**  
You can still use the traditional command line on Windows.

## Installation

As it is tailored to the specific needs of the Amethyst Foundation, `backertrack` is not available for download from cargo.
Therefore, you will have to build it yourself, which is fortunately very easy with cargo.

```
$ git clone https://github.com/Moxinilian/backertrack.git
$ cd backertrack
$ cargo build --release
```

You can grab the output executable in your `target` folder.

Alternatively, you can use `backertrack` directly with `cargo run`:

```
$ git clone https://github.com/Moxinilian/backertrack.git
$ cd backertrack
$ cargo run --release -- [commands]
```

## Usage

`backertrack` can be used from the command line.
To consult the documentation, run:

```
$ backetrack --help
```

You can also use the command line UI by running:

```
$ backertrack -l path/to/ledger/file.json
```
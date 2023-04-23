# sa_ninja_gen

This program generates a ninja build from compile_commands.json that can be used to run the
clang static analyzer (with or without CTU) and clang-tidy.

⚠️ This is a work in progress ⚠️

## Usage

You will need:

  - A compilation database (compile_commands.json). Some build systems, like CMake, will
    generate this for you, but for others you may need to use a tool like [Bear][1].
  - `clang-extdef-mapping` command
  - Both commands from this crate: `sa_ninja_gen` and `merge_extdefs`
  
Most typically, you will want to run this in your build directory because many builds will 
generate files that are required but aren't listed in the compilation database. 

A typical invocation will look like this:

```shell
$ sa_ninja_gen -o ./sa-output compile_commands.json sa.ninja
$ ninja -f sa.ninja
```

The output directory will hold all intermediate files as well as the results of the analysis.

### Using `merge_extdefs` independently

`merge_extdefs` is a separate command that can be used independently from `sa_ninja_gen`. The 
invocation is simple:

```shell
$ merge_extdefs [file1...fileN] [output_file]
# or
$ merge_extdefs @responsefile.txt [output_file]
```

The second form allows passing a text file that contains arguments separated by space or newline
characters. This might be useful on Windows where there is a limitation on the number of characters
that can be passed on the command line. You can supply multiple response file and ordinary 
input files in the same invocation, in any order.

`merge_extdefs` will keep only one definition of a given symbol.

### Environment variables

The following environment variables can be used to influence the behavior of sa_ninja_gen:

  * `CLANG_EXTDEF_MAPPING`: the `clang-extdef-mapping` binary to use
  * `MERGE_EXTDEFS`: the `merge_extdefs` binary to use (this command is supplied by this project)

## Acknowledgments

* The `ninja_syntax` crate is heavily based on [Tobias Hieta's excellent ninja-syntax port][2]
  (used with permission)

[1]: https://github.com/rizsotto/Bear/
[2]: https://github.com/tru/ninja-syntax
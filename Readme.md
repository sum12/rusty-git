Gust (rusty-git)
===============

This is sample implementation of git v1 protocol.

The implementation is not complete and is just for practice.

This cli is more for learning purposes and based on https://wyag.thb.lt/
This repo holds the rust equivalent of python version presented in https://wyag.thb.lt/

The following functionality is implanted :

### init (init subcommand)
This is basic implementation which just creates and empty git repo. 
Does check if .git folder already exists and writes the .git/config file
```
Gust 0.1
Simple rust based git client

USAGE:
    gust [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    cat-file       Provide the content of a repo object
    hash-object    Compute object ID and optionally create a blob from a file
    help           Prints this message or the help of the given subcommand(s)
    init           Create a git repo
```

### Cat-file (cat-file subcommand)
cat-file on appropriate input find and reads the git-object based on the SHA and type.
It will parase the object based on type(blob object, commit object). And print out the 
parsed content.

The commit an zlib compressed compressed object with key-value pairs and size of file.
The cli takes care of correctly parsing the given object and printing the content to stdout.

```
gust-cat-file 
Provide the content of a repo object

USAGE:
    gust cat-file <type> <name>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <type>    Specify type of object [possible values: blob, commit]
    <name>    The object to display

```
### hash-object (hash-object subcommand)
hash-object is reverse of cat-file. It parses the given object file based on the type that
was passed. After parsing it generates an appropriate SHA and prints it to stdout.
```
gust-hash-object 
Compute object ID and optionally create a blob from a file

USAGE:
    gust hash-object [FLAGS] [OPTIONS] <path>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    -w, --write      Actually write the object into the database

OPTIONS:
    -t <type>        Specify type of object [default: blob]  [possible values: blob, commit]

ARGS:
    <path>    Read object from file
```

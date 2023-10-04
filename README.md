# mdBook `webinclude` Preprocessor

[![Crates.io (latest)](https://img.shields.io/crates/v/mdbook-webinclude)](https://crates.io/crates/mdbook-webinclude)

The `webinclude` preprocessor works similar to the built-in
[`include`](https://rust-lang.github.io/mdBook/format/mdbook.html#including-files)
link which inserts (portions of) local files in a section within the book. The same
can be achieved with `webinclude` however instead of specifying a path to the file
a URL can be used. The source of that URL is obtained an further processed like
with `include`.


## Installation & Setup

This preprocessor can be installed with Cargo:

```console
cargo install mdbook-webinclude
```

Add the following line to your `book.toml` file:

```toml
[preprocessor.webinclude]
```

Now you can use the `webinclude` links in your book as described below.


## Usage

### Include Markdown

Because the text is inserted before the book is rendered, Markdown from other
sources can be included. This is especially useful when for instance your book
lays in a different repository than the actual program.

```hbs
{{#webinclude https://example.org/document.md}}
```

<div class="warning">
Keep in mind that header levels remain the same and are not adapted to the current
chapter's header level.
</div>


### Include Text

Text can be inserted as is as well which is less practical unless used within a
code block. You can for example include source code from a different repository
and wrap it in an appropiate code block.

````hbs
```
{{#webinclude https://example.org/main.rs}}
```
````

### Including portions of a file

Often you only need a specific part of the file, e.g. relevant lines for an
example. Four different modes  are supported for partial includes.

```hbs
{{#webinclude https://example.org/document.md 2}}
{{#webinclude https://example.org/document.md :10}}
{{#webinclude https://example.org/document.md 2:}}
{{#webinclude https://example.org/document.md 2:10}}
```

The first command only includes the second line from file. The second
command includes all lines up to line 10, i.e. the lines from 11 till the end of
the file are omitted. The third command includes all lines from line 2, i.e. the
first line is omitted. The last command includes the excerpt of `document.md`
consisting of lines 2 to 10.

To avoid breaking your book when modifying included files, you can also
include a specific section using anchors instead of line numbers.
An anchor is a pair of matching lines. The line beginning an anchor must
match the regex `ANCHOR:\s*[\w_-]+` and similarly the ending line must match
the regex `ANCHOR_END:\s*[\w_-]+`. This allows you to put anchors in
any kind of commented line.

Consider the following file to include:

```rust
/* ANCHOR: all */

// ANCHOR: component
struct Paddle {
    hello: f32,
}
// ANCHOR_END: component

////////// ANCHOR: system
impl System for MySystem { ... }
////////// ANCHOR_END: system

/* ANCHOR_END: all */
```

Then in the book, all you have to do is:

````hbs
Here is a component:
```rust,no_run,noplayground
{{#webinclude file.rs:component}}
```

Here is a system:
```rust,no_run,noplayground
{{#webinclude file.rs:system}}
```

This is the full file.
```rust,no_run,noplayground
{{#webinclude file.rs:all}}
```
````


### Escaping

For the really uncommon case where you want to literally include
`{{#webinclude ...}}` simply prefix it with a backslash (`\`).


### Setting HTTP Headers

HTTP headers can be configured in the `book.toml` file.

Change this line in your `book.toml` file:

```diff
- [preprocessor.webinclude]
+ [preprocessor.webinclude.headers]
```

You can then set the headers as described below.

```toml
[preprocessor.webinclude.headers]
user-agent = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.9; rv:50.0) Gecko/20100101 Firefox/50.0"
accept = "text/plain"
```

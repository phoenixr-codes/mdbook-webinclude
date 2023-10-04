# Introduction

## Test: Entire File

Expected result:

```
Line 1
Line 2
Line 3
Line 4 - ANCHOR: foo
Line 5
Line 6 - ANCHOR: bar
Line 7 - ANCHOR_END: foo
Line 8
Line 9 - ANCHOR_END: bar
```

Rendered result:

```
{{#webinclude https://raw.githubusercontent.com/phoenixr-codes/mdbook-webinclude/master/test_book/example.txt}}
```


## Test: Specific Line

Expected result:

```
Line 3
```

Rendered result:

```
{{#webinclude https://raw.githubusercontent.com/phoenixr-codes/mdbook-webinclude/master/test_book/example.txt 3}}
```


## Test: From Line

Expected result:

```
Line 8
Line 9 - ANCHOR_END: bar
```

Rendered result:

```
{{#webinclude https://raw.githubusercontent.com/phoenixr-codes/mdbook-webinclude/master/test_book/example.txt 8:}}
```


## Test: To Line

Expected result:

```
Line 1
Line 2
Line 3
```

Rendered result:

```
{{#webinclude https://raw.githubusercontent.com/phoenixr-codes/mdbook-webinclude/master/test_book/example.txt :3}}
```


## Test: From Line To Line

Expected result:

```
Line 2
Line 3
```

Rendered result:

```
{{#webinclude https://raw.githubusercontent.com/phoenixr-codes/mdbook-webinclude/master/test_book/example.txt 2:3}}
```


## Test: Anchor

Expected result:

```
Line 5
Line 6 - ANCHOR: bar
```

Rendered result:

```
{{#webinclude https://raw.githubusercontent.com/phoenixr-codes/mdbook-webinclude/master/test_book/example.txt foo}}
```

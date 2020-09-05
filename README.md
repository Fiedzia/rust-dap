# rust-yap - yet another (argument) parser

If you came here looking for argument parser,
turn back and use https://github.com/clap-rs/clap
which is working, reliable solution.

I created yap because I am very opinionated about how arguments should be parsed,
and to experiment with new features clap is missing.

Most notable one is command chain.

Let say you have a program that processes data from multiple sources,
and each source can take verious options.
For example let say we can download data from internet or read local file,
and those sources are expressed as commands:

```
source = 
    file PATH
    web [OPTIONS] URL
```

and we want to accept multiple sources:

```
import file /some/file file /some/other/file web http://somesite.com
```

Clap doesn't support that, I needed this, so I wrote my own parser.

This is work in progress, and its not even aimed at being as mature as clap,
but it will do what I need.

# rust-dap - declarative argument parser

rust-dap ~~is~~ *will be* (for now its work in progress) declarative argument parser

The idea is to have declarative definition of your arguments,
from which Rust code will be generated.

rust-dap uses power of lalrpop to parse argument definition file
and is capable of handling program arguments without adding unnecessary complexity to your code
(by moving it to your build process).

I created it because structopt was generating hard to read code,
was missing certain features and was cumbersome to use for more complex cases.

Rust-dap should handle almost anything you throw at it.

For example this declaration:

```python
verbose = BooleanFlag(
    name = 'verbose',
    default_value = true,
    short = 'v',
    long  = 'verbose',
    help  = 'Be verbose',
)

program_options = Options(flags=[verbose])


order_command = Command(
    name = 'order'
)

status_argument = Argument(
    type = String,
    amount = 1
)

status_command = Command(
    name = 'status',
    arguments = [status_argument]
)


order_cmdline = CmdLine(
    syntax = 'program_options order_command items=String*'
)

order_status_cmdline = CmdLine(
    syntax = 'program_options status_command'
)


main = EnumCmdLine(
    enums = [order_cmdline, order_status_cmdline]
)

```
will generate rust parser that can handle following invocations:

```bash
    ./food order pizza ravioli
    ./food status order1 
```

while all you need to do in your code will be to implement relevant functions:

```rust

    pub fn handle_order_cmdline(program_options, order_command, items) {...}
    pub fn handle_order_status_cmdline(program_options, status_command) {...}

```




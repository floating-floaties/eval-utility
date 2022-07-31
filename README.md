# Eval Utility

Wrapper function of the [resolver crate](https://crates.io/crates/resolver). Provides python-like built-in functions.

## Crate

```toml
[dependencies]
resolver = "^0.1"
eval-utility = "^0.1"
```


## Example

See test cases in [`lib.rs`](https://github.com/floating-floaties/eval-utility/blob/main/src/lib.rs#L567) for more examples.

```rust

use eval_utility::eval_wrapper::{expr_wrapper, EvalConfig};

fn main () {
    let expression = "float('42.42') == 42.42";
    let expr = expr_wrapper(
        resolver::Expr::new(expression),
        EvalConfig::default(),
    );

    println!("{:?}", expr.exec());
}

```


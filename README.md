# Eval Utility

Wrapper function of the [resolver crate](https://crates.io/crates/resolver). Provides python-like built-in functions.

## Crate

```toml
[dependencies]
resolver = "0.1"
eval-utility = "0.2"
```


## Example

See test cases in [`lib.rs`](https://github.com/floating-floaties/eval-utility/blob/main/src/lib.rs#L567) for more examples.

```rust
use resolver::{Expr, to_value};
use eval_utility::eval_wrapper::{expr_wrapper, EvalConfig};

fn main () {
    let expression = "float('42.42') == 42.42";
    let expected = true;
    
    let expr = expr_wrapper(
        Expr::new(expression),
        EvalConfig {
            include_maths: true,
            include_datetime: true,
            include_cast: true,
            include_regex: true,
        },
    );

    match expr.exec() {
         Ok(value) => {
            assert_eq!(value, to_value(expected));
        },
        Err(err) => {
            panic!("err={err:?}");
        }
    };
    // ...
}
```


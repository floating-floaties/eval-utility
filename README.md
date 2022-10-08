# Eval Utility

Wrapper function of the [resolver crate](https://crates.io/crates/resolver). Provides python-like built-in functions.

## Crate

```toml
[dependencies]
resolver = "0.1"
eval-utility = "0.2"
```


## Example

See test cases in [`lib.rs`](https://github.com/floating-floaties/eval-utility/blob/main/src/lib.rs#L484) for more examples.

```rust
use eval_utility::eval_wrapper::{EvalConfig, ExprWrapper};

fn main() {
    let expression = "float('42.42') == 42.42";
    let expected = true;


    let expr = ExprWrapper::new(expression)
        // .config(Default::default())
        .config(EvalConfig { // same as Default::default() ^
            include_maths: true,
            include_regex: true,
            include_datetime: true,
            include_cast: true,
        })
        .init();

    match expr.exec() {
        Ok(value) => {
            assert_eq!(value, expected);
        }
        Err(err) => {
            panic!("err={err:?}");
        }
    };
    // ...
}
```


# Eval Utility

Wrapper function of the [eval crate](https://crates.io/crates/eval). Provides python-like built-in functions.


## Example

See test cases in `lib.rs` for more examples.

```rust

use eval_utility::eval_wrapper::{expr_wrapper, EvalConfig};

fn main () {
    let expression = "float('42.42') == 42.42";
    let expr = expr_wrapper(
        eval::Expr::new(expression),
        EvalConfig::default(),
    );

    println!("{:?}", expr.exec());
}

```


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
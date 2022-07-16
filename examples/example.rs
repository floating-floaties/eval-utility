use eval_utility::eval_wrapper::{expr_wrapper, EvalConfig};

fn main () {
    let expression = "float('42.42') == 42.42";
    let expr = expr_wrapper(
        resolver::Expr::new(expression),
        EvalConfig::default(),
    );

    let result = expr.exec();

    println!("\"{}\" resolved to {:?}", expression, result);
}

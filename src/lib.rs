#![doc = include_str ! ("./../README.md")]
#![forbid(unsafe_code)]

pub mod types {
    pub type Expr = resolver::Expr;
    pub type Value = resolver::Value;

    pub fn to_value<S: serde::Serialize>(v: S) -> Value {
        resolver::to_value(v)
    }
}

pub mod template {
    use lazy_static::lazy_static;
    use regex::Regex;

    use crate::types::*;

    lazy_static! {
        static ref CONDITION_PATTERN: Regex = Regex::new(r"(<\?([^\?]*)\?>)").unwrap();
        static ref CONTEXT_SYM: String = String::from("$");
    }

    pub fn resolve_template(
        template: String,
        context: Value,
    ) -> Result<String, resolver::Error> {
        let mut map = hashbrown::HashMap::<String, String>::new();
        for cap in CONDITION_PATTERN.captures_iter(&*template) {
            let a = &cap[1];
            let b = cap[2].trim();
            if !b.is_empty() {
                let expr = resolver::Expr::new(b)
                    .value(CONTEXT_SYM.to_string(), &context);
                let value = expr.exec()?;
                let value_str = match value {
                    Value::Null => "null".into(),
                    Value::Bool(boolean) => boolean.to_string(),
                    Value::Number(number) => number.to_string(),
                    Value::String(string) => string,
                    Value::Array(arr) => serde_json::to_string(&arr)
                        .unwrap_or_else(|_| "null".into()),
                    Value::Object(obj) => serde_json::to_string(&obj)
                        .unwrap_or_else(|_| "null".into())
                };
                map.insert(a.to_string(), value_str);
            } else {
                map.insert(a.to_string(), "".into());
            }
        }

        let mut result = template;
        for (key, value) in map.iter() {
            // TODO(Dustin): Replace by range?
            result = result.replace(key, value);
        }

        Ok(result)
    }
}

pub mod eval_wrapper {
    use std::sync::{Arc, Mutex};
    use chrono::{Datelike, Timelike};
    use lazy_static::lazy_static;
    use resolver::{to_value, Expr};
    use regex::Regex;
    use inflection_rs::inflection::Inflection;

    use crate::types::*;

    macro_rules! substr {
        ($str:expr, $start_pos:expr) => {{
            substr!($str, $start_pos, $str.len())
        }};

        ($str:expr, $start_pos:expr, $end_pos:expr) => {{
            substr!($str, $start_pos, $end_pos - $start_pos, true)
        }};

        ($str:expr, $start_pos:expr, $take_count:expr, $use_take:expr) => {{
            &$str
                .chars()
                .skip($start_pos)
                .take($take_count)
                .collect::<String>()
        }};
    }

    lazy_static! {
        static ref INFLECTION: Arc<Mutex<Inflection>> = Arc::new(Mutex::new(Inflection::default()));
    }

//     let g = Arc::clone(&INFLECTION);
//     let lock = g.lock();
//     let v = match lock {
//     Ok(mut inf) => {
// inf.pluralize("hello")
// },
//     Err(err) => {
// log::error!("ERROR: Failed to acquire inflection resource lock '{:?}'", err);
// "".to_string()
// }
// };

    #[derive(Debug, Clone)]
    pub struct EvalConfig {
        pub include_maths: bool,
        pub include_datetime: bool,
        pub include_cast: bool,
        pub include_regex: bool,
    }

    impl EvalConfig {
        pub fn any(&self) -> bool {
            self.include_maths
                || self.include_datetime
                || self.include_cast
                || self.include_regex
        }
    }

    impl Default for EvalConfig {
        fn default() -> Self {
            Self {
                include_maths: true,
                include_datetime: true,
                include_cast: true,
                include_regex: true,
            }
        }
    }

    fn value_to_string(val: &Value) -> String {
        match val {
            Value::Number(x) => x.as_f64().unwrap().to_string(),
            Value::Bool(x) => x.to_string(),
            Value::String(x) => x.to_string(),
            Value::Array(x) => serde_json::to_string(x)
                .unwrap_or_else(|_| "null".into()),
            Value::Object(x) => serde_json::to_string(x)
                .unwrap_or_else(|_| "null".into()),
            _ => String::from("null"),
        }
    }

    pub fn math_consts() -> Value {
        serde_json::json!{{
            "MIN_INT": i64::MIN,
            "MAX_INT": i64::MAX,
            "MAX_FLOAT": f64::MAX,
            "MIN_FLOAT": f64::MIN,
            "INC": f64::NAN,
            "NOT_A_NUMBER": f64::NAN,
            "INFINITE": f64::INFINITY,
            "NEG_INFINITE": f64::NEG_INFINITY,
            "E": std::f64::consts::E,
            "FRAC_1_SQRT_2": std::f64::consts::FRAC_1_SQRT_2,
            "FRAC_2_SQRT_PI": std::f64::consts::FRAC_2_SQRT_PI,
            "FRAC_1_PI": std::f64::consts::FRAC_1_PI,
            "FRAC_PI_2": std::f64::consts::FRAC_PI_2,
            "FRAC_PI_3": std::f64::consts::FRAC_PI_3,
            "FRAC_PI_4": std::f64::consts::FRAC_PI_4,
            "FRAC_PI_6": std::f64::consts::FRAC_PI_6,
            "FRAC_PI_8": std::f64::consts::FRAC_PI_8,
            "LN_2": std::f64::consts::LN_2,
            "LN_10": std::f64::consts::LN_10,
            "LOG2_10": std::f64::consts::LOG2_10,
            "LOG2_E": std::f64::consts::LOG2_E,
            "LOG10_2": std::f64::consts::LOG10_2,
            "LOG10_E": std::f64::consts::LOG10_E,
            "PI": std::f64::consts::PI,
            "SQRT_2": std::f64::consts::SQRT_2,
            "TAU": std::f64::consts::TAU,
        }}
    }

    #[derive(Clone)]
    pub struct ExprWrapper {
        expr: Expr,
        config: EvalConfig,
    }

    impl ExprWrapper {
        pub fn new<S: AsRef<str>>(expression: S) -> ExprWrapper {
            ExprWrapper {
                expr: Expr::new(expression.as_ref()),
                config: Default::default(),
            }
        }

        pub fn config(mut self, config: EvalConfig) -> ExprWrapper {
            self.config = config;
            self
        }

        pub fn init(mut self) -> ExprWrapper {
            self.expr = expr_wrapper(self.expr.clone(), self.config.clone());
            self
        }

        pub fn exec(&self) -> Result<Value, resolver::Error> {
            self.expr.exec()
        }
    }

    /// This function is DEPRECATED see README.md for new usage.
    #[deprecated]
    pub fn expr_wrapper(exp: Expr, config: EvalConfig) -> Expr {
        if !config.any() {
            return exp;
        }

        let mut result = exp;

        if config.include_cast {
            result = result
                .function("int", |value| {
                    if value.is_empty() {
                        return Ok(to_value(0_i64));
                    }
                    let v = match value.get(0) {
                        None => to_value(0),
                        Some(value) => value.to_owned(),
                    };

                    let num: i64 = match v {
                        Value::Number(x) => {
                            if x.is_f64() {
                                x.as_f64().unwrap_or(0_f64) as i64
                            } else {
                                x.as_i64().unwrap_or(0)
                            }
                        }
                        Value::Bool(x) => {
                            if x {
                                1
                            } else {
                                0
                            }
                        }
                        Value::String(x) => atoi(x),
                        _ => 0,
                    };
                    Ok(to_value(num))
                })
                .function("float", |value| {
                    if value.is_empty() {
                        return Ok(to_value(f64::NAN));
                    }
                    let v = match value.get(0) {
                        None => to_value(0_f64),
                        Some(value) => value.to_owned(),
                    };
                    let num: f64 = match v {
                        Value::Number(x) => x.as_f64().unwrap_or(0_f64),
                        Value::Bool(x) => {
                            if x {
                                1.0
                            } else {
                                0.0
                            }
                        }
                        Value::String(x) => match x.parse::<f64>() {
                            Ok(x) => x,
                            _ => f64::NAN,
                        },
                        _ => f64::NAN,
                    };

                    Ok(to_value(num))
                })
                .function("bool", |value| {
                    if value.is_empty() {
                        return Ok(to_value(false));
                    }
                    let v = match value.get(0) {
                        None => to_value(false),
                        Some(value) => value.to_owned(),
                    };

                    let result: bool = match v {
                        Value::Number(x) => x.as_f64().unwrap_or(0_f64) != 0.0,
                        Value::Bool(x) => x,
                        Value::String(x) => !x.is_empty(),
                        Value::Array(x) => !x.is_empty(),
                        Value::Object(x) => !x.is_empty(),
                        _ => false,
                    };

                    Ok(to_value(result))
                })
                .function("str", |value| {
                    if value.is_empty() {
                        return Ok(to_value("".to_string()));
                    }
                    let v = match value.get(0) {
                        None => to_value("".to_string()),
                        Some(value) => value.to_owned(),
                    };

                    let result: String = match v {
                        Value::Number(x) => {
                            if x.is_f64() {
                                x.as_f64().unwrap_or(0_f64).to_string()
                            } else {
                                x.as_i64().unwrap_or(0_i64).to_string()
                            }
                        }
                        Value::Bool(x) => x.to_string(),
                        Value::String(x) => x,
                        Value::Array(x) => serde_json::to_string(&x)
                            .unwrap_or_else(|_| "null".to_string()),
                        Value::Object(x) => serde_json::to_string(&x)
                            .unwrap_or_else(|_| "null".to_string()),
                        _ => "null".to_string(),
                    };
                    Ok(to_value(result))
                });
        }

        if config.include_maths {
            result = result
                .value("maths", math_consts())
                .value("NAN", to_value(f64::NAN))
                .value("INFINITY", to_value(f64::INFINITY))
                .value("NEG_INFINITY", to_value(f64::NEG_INFINITY));
        }

        if config.include_regex {
            result = result.function("is_match", |value| {
                if value.len() < 2 {
                    return Ok(to_value(false));
                }

                let v = value.get(0).unwrap();
                let pattern = value.get(1).unwrap().as_str().unwrap();

                let value: String = value_to_string(v);

                let prog = Regex::new(pattern).unwrap();
                let is_match = prog.is_match(&value);
                Ok(to_value(is_match))
            }).function("extract", |value| {
                if value.len() < 2 {
                    return Ok(to_value(false));
                }

                let v = value
                    .get(0).expect("missing first positional argument (string)");
                let pattern = value
                    .get(1).expect("missing second positional argument (pattern)")
                    .as_str().expect("second positional arguments needs to be a string");

                let value: String = value_to_string(v);
                let prog = Regex::new(pattern).unwrap();
                match prog.find(&value) {
                    None => Ok(to_value("".to_string())),
                    Some(m) => {
                        let (start, end) = (m.start(), m.end());
                        Ok(to_value(substr!(value, start, end)))
                    }
                }
            });
        }

        if config.include_datetime {
            result = result
                .function("get_day", |values| {
                    let current_time = eval_tz_parse_args(values, 1);
                    Ok(to_value(current_time.date().day()))
                })
                .function("get_month", |values| {
                    let current_time = eval_tz_parse_args(values, 1);
                    Ok(to_value(current_time.date().month()))
                })
                .function("get_year", |values| {
                    let current_time = eval_tz_parse_args(values, 1);
                    Ok(to_value(current_time.date().year()))
                })
                .function("get_weekday", |values| {
                    let current_time = eval_tz_parse_args(values, 1);
                    Ok(to_value(
                        current_time.date().weekday().number_from_monday(),
                    ))
                })
                .function("is_weekday", |values| {
                    let current_time = eval_tz_parse_args(values, 1);
                    let weekday = current_time.date().weekday().number_from_monday();
                    Ok(to_value(weekday < 6))
                })
                .function("is_weekend", |values| {
                    let current_time = eval_tz_parse_args(values, 1);
                    let weekday = current_time.date().weekday();
                    let weekends = [chrono::Weekday::Sat, chrono::Weekday::Sun];
                    Ok(to_value(weekends.contains(&weekday)))
                })
                .function("get_time", |extract| {
                    if extract.len() < 2 {
                        let t = now("_".to_owned());
                        return Ok(to_value(t.hour()));
                    }

                    let v: String = match extract.get(1).unwrap() {
                        Value::Number(x) => {
                            if x.is_f64() {
                                x.as_f64().unwrap().to_string()
                            } else if x.is_i64() {
                                x.as_i64().unwrap().to_string()
                            } else if x.is_u64() {
                                x.as_u64().unwrap().to_string()
                            } else {
                                x.to_string()
                            }
                        }
                        Value::Bool(x) => x.to_string(),
                        Value::String(x) => x.to_string(),
                        Value::Array(x) => serde_json::to_string(x).unwrap(),
                        Value::Object(x) => serde_json::to_string(x).unwrap(),
                        _ => String::from("null"),
                    };

                    let dt = eval_tz_parse_args(extract, 2);
                    let current_time = dt.time();

                    let result = match v.as_str() {
                        "h" | "hour" | "hours" => current_time.hour(),
                        "m" | "minute" | "minutes" => current_time.minute(),
                        "s" | "second" | "seconds" => current_time.second(),
                        _ => current_time.hour(),
                    };
                    Ok(to_value(result))
                });
        }

        result

        // TODO: is_nan(n), is_min_int(n), is_int_max(n), includes(arr)
        // TODO: min(arr), max(arr), abs(n), pow(n, p), sum(arr), reverse(arr), sort(arr), unique(arr)
    }

    fn eval_tz_parse_args(
        arguments: Vec<Value>,
        min_args: usize,
    ) -> chrono::DateTime<chrono_tz::Tz> {
        let default_tz = "_".to_owned();
        if arguments.is_empty() || arguments.len() < min_args {
            log::warn!("No arguments");
            return now(default_tz);
        }

        let v: Option<String> = match arguments.get(0).unwrap() {
            Value::String(x) => Some(x.to_string()),
            _ => None,
        };

        match v {
            None => {
                log::warn!("Invalid Timezone");
                now(default_tz)
            }
            Some(timezone) => now(timezone)
        }
    }

    fn now(tz: String) -> chrono::DateTime<chrono_tz::Tz> {
        chrono::offset::Utc::now()
            .with_timezone(&str_to_tz(tz))
    }

    fn str_to_tz(timezone: String) -> chrono_tz::Tz {
        match timezone.parse() {
            Ok(tz) => tz,
            Err(_err) => {
                log::warn!("Defaulted to UTC timezone");
                chrono_tz::UTC
            }
        }
    }

    fn atoi(s: String) -> i64 {
        let mut item = s
            .trim()
            .split(char::is_whitespace)
            .next()
            .unwrap_or("")
            .split(char::is_alphabetic)
            .next()
            .unwrap_or("");

        let mut end_idx = 0;
        for (pos, c) in item.chars().enumerate() {
            if pos == 0 {
                continue;
            }

            if !c.is_alphanumeric() {
                end_idx = pos;
                break;
            }
        }

        if end_idx > 0 {
            item = &item[0..end_idx];
        }

        let result = item.parse::<i64>();
        match result {
            Ok(v) => v,
            Err(error) => match error.kind() {
                std::num::IntErrorKind::NegOverflow => i64::MIN,
                std::num::IntErrorKind::PosOverflow => i64::MAX,
                std::num::IntErrorKind::InvalidDigit => {
                    let result = item.parse::<f64>();
                    match result {
                        Ok(v) => v.round() as i64,
                        _ => 0,
                    }
                }
                _ => 0,
            },
        }
    }
}


#[cfg(test)]
mod eval {
    use chrono::offset::Utc as Date;
    use chrono::{Datelike, Timelike};
    use resolver::to_value;
    use serde_json::json;

    use crate::{eval_wrapper, template};

    #[derive(Default)]
    struct Spec;

    impl Spec {
        pub fn eval<S: AsRef<str>>(&self, expression: S) -> resolver::Value {
            let expr = eval_wrapper::expr_wrapper(
                resolver::Expr::new(expression.as_ref().to_owned()),
                Default::default(),
            );
            let result = expr.exec();

            if result.is_err() {
                panic!(
                    "Failed to parse expression: \"{}\" {:?}",
                    expression.as_ref().to_owned(),
                    result
                )
            }

            result.unwrap()
        }
    }

    #[test]
    fn maths_consts() {
        let user_spec = Spec::default();
        assert_eq!(user_spec.eval("NAN"), to_value(f64::NAN));
        assert_eq!(user_spec.eval("INFINITY"), to_value(f64::INFINITY));
        assert_eq!(user_spec.eval("NEG_INFINITY"), to_value(f64::NEG_INFINITY));
        assert_eq!(user_spec.eval("maths.MAX_INT"), to_value(i64::MAX));
        assert_eq!(user_spec.eval("maths.MAX_FLOAT"), to_value(f64::MAX));
        assert_eq!(user_spec.eval("maths.MIN_FLOAT"), to_value(f64::MIN));
        assert_eq!(user_spec.eval("maths.INC"), to_value(f64::NAN));
        assert_eq!(user_spec.eval("maths.NOT_A_NUMBER"), to_value(f64::NAN));
        assert_eq!(user_spec.eval("maths.INFINITE"), to_value(f64::INFINITY));
        assert_eq!(user_spec.eval("maths.NEG_INFINITE"), to_value(f64::NEG_INFINITY));
        assert_eq!(user_spec.eval("maths.E"), to_value(std::f64::consts::E));
        assert_eq!(user_spec.eval("maths.FRAC_1_SQRT_2"), to_value(std::f64::consts::FRAC_1_SQRT_2));
        assert_eq!(user_spec.eval("maths.FRAC_2_SQRT_PI"), to_value(std::f64::consts::FRAC_2_SQRT_PI));
        assert_eq!(user_spec.eval("maths.FRAC_1_PI"), to_value(std::f64::consts::FRAC_1_PI));
        assert_eq!(user_spec.eval("maths.FRAC_PI_2"), to_value(std::f64::consts::FRAC_PI_2));
        assert_eq!(user_spec.eval("maths.FRAC_PI_3"), to_value(std::f64::consts::FRAC_PI_3));
        assert_eq!(user_spec.eval("maths.FRAC_PI_4"), to_value(std::f64::consts::FRAC_PI_4));
        assert_eq!(user_spec.eval("maths.FRAC_PI_6"), to_value(std::f64::consts::FRAC_PI_6));
        assert_eq!(user_spec.eval("maths.FRAC_PI_8"), to_value(std::f64::consts::FRAC_PI_8));
        assert_eq!(user_spec.eval("maths.LN_2"), to_value(std::f64::consts::LN_2));
        assert_eq!(user_spec.eval("maths.LN_10"), to_value(std::f64::consts::LN_10));
        assert_eq!(user_spec.eval("maths.LOG2_10"), to_value(std::f64::consts::LOG2_10));
        assert_eq!(user_spec.eval("maths.LOG2_E"), to_value(std::f64::consts::LOG2_E));
        assert_eq!(user_spec.eval("maths.LOG10_2"), to_value(std::f64::consts::LOG10_2));
        assert_eq!(user_spec.eval("maths.LOG10_E"), to_value(std::f64::consts::LOG10_E));
        assert_eq!(user_spec.eval("maths.PI"), to_value(std::f64::consts::PI));
        assert_eq!(user_spec.eval("maths.SQRT_2"), to_value(std::f64::consts::SQRT_2));
        assert_eq!(user_spec.eval("maths.TAU"), to_value(std::f64::consts::TAU));
    }

    #[test]
    fn literal() {
        let user_spec = Spec::default();

        assert_eq!(user_spec.eval("42"), 42);
        assert_eq!(user_spec.eval("0-42"), -42);
        assert_eq!(user_spec.eval("true"), true);
        assert_eq!(user_spec.eval("false"), false);
        assert_eq!(user_spec.eval("\"42\""), "42");
        assert_eq!(user_spec.eval("'42'"), "42");
        assert_eq!(user_spec.eval("array(42, 42)"), to_value(vec![42; 2]));
        assert_eq!(user_spec.eval("array()"), to_value(vec![0; 0]));
        assert_eq!(user_spec.eval("0..5"), to_value(vec![0, 1, 2, 3, 4]));
    }

    #[test]
    fn _str() {
        let user_spec = Spec::default();
        assert_eq!(user_spec.eval("str(42)"), "42");
        assert_eq!(user_spec.eval("str(42.42)"), "42.42");
        assert_eq!(user_spec.eval("str(true)"), "true");
        assert_eq!(user_spec.eval("str(array(42, 42))"), to_value("[42,42]"));
        assert_eq!(user_spec.eval("str(array())"), to_value("[]"));
        assert_eq!(user_spec.eval("str(null)"), to_value("null"));
    }

    #[test]
    fn bool() {
        let user_spec = Spec::default();

        assert_eq!(user_spec.eval("bool(1)"), true);
        assert_eq!(user_spec.eval("bool(1.0)"), true);
        assert_eq!(user_spec.eval("bool(0)"), false);
        assert_eq!(user_spec.eval("bool(0.0)"), false);
        assert_eq!(user_spec.eval("bool(true)"), true);
        assert_eq!(user_spec.eval("bool(false)"), false);

        assert_eq!(user_spec.eval("bool(42)"), true);
        assert_eq!(user_spec.eval("bool(42.42)"), true);
        assert_eq!(user_spec.eval("bool(0-42)"), true);
        assert_eq!(user_spec.eval("bool(0-42.42)"), true);

        assert_eq!(user_spec.eval("bool('')"), false);
        assert_eq!(user_spec.eval("bool(\"\")"), false);
        assert_eq!(user_spec.eval("bool('42')"), true);
        assert_eq!(user_spec.eval("bool(\"42\")"), true);

        assert_eq!(user_spec.eval("bool(array(42, 42))"), true);
        assert_eq!(user_spec.eval("bool(array())"), false);
        assert_eq!(user_spec.eval("bool(0..42)"), true);
        assert_eq!(user_spec.eval("bool(0..0)"), false);
        assert_eq!(user_spec.eval("bool(null)"), false);
    }

    #[test]
    fn float() {
        let user_spec = Spec::default();
        assert_eq!(user_spec.eval("float(42)"), 42.0);
        assert_eq!(user_spec.eval("float(42.42)"), 42.42);
        assert_eq!(user_spec.eval("float('42.42')"), 42.42);
        assert_eq!(user_spec.eval("float('42')"), 42.0);
        assert_eq!(user_spec.eval("float(true)"), 1.0);
        assert_eq!(user_spec.eval("float(false)"), 0.0);
        assert_eq!(user_spec.eval("float('')"), to_value(f64::NAN));
        assert_eq!(
            user_spec.eval("float('not a num')"),
            to_value(f64::NAN)
        );
        assert_eq!(user_spec.eval("float(ctx)"), to_value(f64::NAN));
        assert_eq!(
            user_spec.eval("float(array(42, 42))"),
            to_value(f64::NAN)
        );
        assert_eq!(user_spec.eval("float(0..42)"), to_value(f64::NAN));
        assert_eq!(user_spec.eval("float(null)"), to_value(f64::NAN));
    }

    #[test]
    fn int() {
        let user_spec = Spec::default();
        assert_eq!(user_spec.eval("int(42)"), 42);
        assert_eq!(user_spec.eval("int(42.42)"), 42);
        assert_eq!(user_spec.eval("int('42.42')"), 42);
        assert_eq!(user_spec.eval("int('42')"), 42);
        assert_eq!(user_spec.eval("int(true)"), 1);
        assert_eq!(user_spec.eval("int(false)"), 0);
        assert_eq!(user_spec.eval("int('')"), 0);
        assert_eq!(user_spec.eval("int('not a num')"), 0);
        assert_eq!(user_spec.eval("int(ctx)"), 0);
        assert_eq!(user_spec.eval("int(array(42, 42))"), 0);
        assert_eq!(user_spec.eval("int(0..42)"), 0);
        assert_eq!(user_spec.eval("int(null)"), 0);
    }

    #[test]
    fn day() {
        let user_spec = Spec::default();
        let date = Date::now().date();
        let day = date.day();

        assert_eq!(user_spec.eval("get_day()"), day);
        assert_eq!(user_spec.eval("get_day('_')"), day);
    }

    #[test]
    fn month() {
        let user_spec = Spec::default();
        let date = Date::now().date();
        let month = date.month();

        assert_eq!(user_spec.eval("get_month()"), month);
        assert_eq!(user_spec.eval("get_month('_')"), month);
    }

    #[test]
    fn year() {
        let user_spec = Spec::default();
        let date = Date::now().date();
        let year = date.year();
        assert_eq!(user_spec.eval("get_year()"), year);
        assert_eq!(user_spec.eval("get_year('_')"), year);
    }

    #[test]
    fn weekday() {
        let user_spec = Spec::default();
        let weekday_num = Date::now().weekday().number_from_monday();
        assert_eq!(user_spec.eval("get_weekday('_')"), weekday_num);
        assert_eq!(user_spec.eval("is_weekday('_')"), weekday_num < 6);

        assert_eq!(user_spec.eval("get_weekday()"), weekday_num);
        assert_eq!(user_spec.eval("is_weekday()"), weekday_num < 6);
    }

    #[test]
    fn time() {
        let user_spec = Spec::default();
        assert_eq!(user_spec.eval("get_time('_', 'h')"), Date::now().time().hour());
        assert_eq!(user_spec.eval("get_time('_', 'm')"), Date::now().time().minute());
        assert_eq!(user_spec.eval("get_time('_', 's')"), Date::now().time().second());

        assert_eq!(user_spec.eval("get_time('_', 'hour')"), Date::now().time().hour());
        assert_eq!(
            user_spec.eval("get_time('_', 'minute')"),
            Date::now().time().minute()
        );
        assert_eq!(
            user_spec.eval("get_time('_', 'second')"),
            Date::now().time().second()
        );

        assert_eq!(user_spec.eval("get_time('_', 'hours')"), Date::now().time().hour());
        assert_eq!(user_spec.eval("get_time()"), Date::now().time().hour());
        assert_eq!(
            user_spec.eval("get_time('_', 'minutes')"),
            Date::now().time().minute()
        );
        assert_eq!(
            user_spec.eval("get_time('_', 'seconds')"),
            Date::now().time().second()
        );
    }

    #[test]
    fn is_match() {
        let user_spec = Spec::default();
        assert_eq!(user_spec.eval("is_match('http', '^https?$')"), to_value(true));
        assert_eq!(user_spec.eval("is_match('http', 'https')"), to_value(false));
        assert_eq!(user_spec.eval("is_match('http://', '^udp://')"), to_value(false));
        assert_eq!(user_spec.eval("is_match('http://', '^(https?|wss?)://$')"), to_value(true));
        assert_eq!(user_spec.eval(r"is_match('2014-01-01', '^\d{4}-\d{2}-\d{2}$')"), to_value(true));
    }

    #[test]
    fn extract() {
        let user_spec = Spec::default();
        assert_eq!(user_spec.eval("extract('http://www.floa', 'https?://')"), "http://");
        assert_eq!(user_spec.eval("extract('foo', 'bar')"), "");
    }

    #[test]
    fn template_engine() {
        let context = json! {{
            "name": "Kar",
            "location": "foo-bar",
            "some": {
                "deep": {
                    "value": 42
                }
            }
        }};

        assert_eq!(
            template::resolve_template(
                "Hi, my name is <? $.name ?> and I live in <? $.location ?> <? $.some.deep.value ?>".to_string(),
                context.clone(),
            ).expect("Failed to resolve template"),
            "Hi, my name is Kar and I live in foo-bar 42".to_string()
        );

        assert_eq!(
            template::resolve_template(
                "Hi, my name is Kar and I live in foo-bar 42".to_string(),
                context.clone(),
            ).expect("Failed to resolve template"),
            "Hi, my name is Kar and I live in foo-bar 42".to_string()
        );

        assert_eq!(
            template::resolve_template(
                "".to_string(),
                context.clone(),
            ).expect("Failed to resolve template"),
            "".to_string()
        );

        assert_eq!(
            template::resolve_template(
                "Hello, <? ?>".to_string(),
                context.clone(),
            ).expect("Failed to resolve template"),
            "Hello, ".to_string(),
        );
    }
}

#![doc = include_str!("./../README.md")]
pub mod eval_wrapper {
    use chrono::{Datelike, TimeZone, Timelike};
    use resolver::{to_value, Expr, Value};
    use regex::Regex;

    pub mod consts {
        pub mod tz {
            pub const US_ALASKA: &str = "US/Alaska";
            pub const US_ALEUTIAN: &str = "US/Aleutian";
            pub const US_ARIZONA: &str = "US/Arizona";
            pub const US_CENTRAL: &str = "US/Central";
            pub const US_EASTINDIANA: &str = "US/EastIndiana";
            pub const US_EASTERN: &str = "US/Eastern";
            pub const US_HAWAII: &str = "US/Hawaii";
            pub const US_INDIANA_STARKE: &str = "US/IndianaStarke";
            pub const US_MICHIGAN: &str = "US/Michigan";
            pub const US_MOUNTAIN: &str = "US/Mountain";
            pub const US_PACIFIC: &str = "US/Pacific";
            pub const US_SAMOA: &str = "US/Samoa";
        }
    }

    #[derive(Debug, Clone)]
    pub struct EvalConfig {
        pub include_maths: bool,
        pub include_datetime: bool,
        pub include_cast: bool,
        pub include_regex: bool,
    }

    impl EvalConfig {
        pub fn default() -> Self {
            EvalConfig {
                include_maths: true,
                include_datetime: true,
                include_cast: true,
                include_regex: true,
            }
        }

        pub fn any(&self) -> bool {
            return self.include_maths
                || self.include_datetime
                || self.include_cast
                || self.include_regex;
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum TypeOfString {
        INT64,
        F64,
        BOOLEAN,
        STRING,
        ARRAY,
        OBJECT,
        NULL,
    }

    impl TypeOfString {
        pub fn value(&self) -> String {
            match *self {
                TypeOfString::INT64 => "INTEGER".to_owned(),
                TypeOfString::F64 => "FLOAT".to_owned(),
                TypeOfString::BOOLEAN => "BOOLEAN".to_owned(),
                TypeOfString::STRING => "STRING".to_owned(),
                TypeOfString::ARRAY => "ARRAY".to_owned(),
                TypeOfString::OBJECT => "OBJECT".to_owned(),
                TypeOfString::NULL => "NULL".to_owned(),
            }
        }

        pub fn from_value<S: AsRef<str>>(value: S) -> TypeOfString {
            match value.as_ref().to_uppercase().trim() {
                "INTEGER" => TypeOfString::INT64,
                "FLOAT" => TypeOfString::F64,
                "BOOLEAN" => TypeOfString::BOOLEAN,
                "STRING" => TypeOfString::STRING,
                "ARRAY" => TypeOfString::ARRAY,
                "OBJECT" => TypeOfString::OBJECT,
                _ => TypeOfString::NULL,
            }
        }
    }

    pub fn math_consts() -> Vec<(String, (String, TypeOfString))> {
        return vec![
            (
                "MIN_INT".to_string(),
                (std::i64::MIN.to_string(), TypeOfString::INT64),
            ),
            (
                "MAX_INT".to_string(),
                (std::i64::MAX.to_string(), TypeOfString::INT64),
            ),
            (
                "MAX_FLOAT".to_string(),
                (std::f64::MAX.to_string(), TypeOfString::F64),
            ),
            (
                "MIN_FLOAT".to_string(),
                (std::f64::MIN.to_string(), TypeOfString::F64),
            ),
            (
                "NAN".to_string(),
                (std::f64::NAN.to_string(), TypeOfString::F64),
            ),
            (
                "INFINITY".to_string(),
                (std::f64::INFINITY.to_string(), TypeOfString::F64),
            ),
            (
                "NEG_INFINITY".to_string(),
                (std::f64::NEG_INFINITY.to_string(), TypeOfString::F64),
            ),
            (
                "E".to_string(),
                (std::f64::consts::E.to_string(), TypeOfString::F64),
            ),
            (
                "FRAC_1_SQRT_2".to_string(),
                (
                    std::f64::consts::FRAC_1_SQRT_2.to_string(),
                    TypeOfString::F64,
                ),
            ),
            (
                "FRAC_2_SQRT_PI".to_string(),
                (
                    std::f64::consts::FRAC_2_SQRT_PI.to_string(),
                    TypeOfString::F64,
                ),
            ),
            (
                "FRAC_1_PI".to_string(),
                (std::f64::consts::FRAC_1_PI.to_string(), TypeOfString::F64),
            ),
            (
                "FRAC_PI_2".to_string(),
                (std::f64::consts::FRAC_PI_2.to_string(), TypeOfString::F64),
            ),
            (
                "FRAC_PI_3".to_string(),
                (std::f64::consts::FRAC_PI_3.to_string(), TypeOfString::F64),
            ),
            (
                "FRAC_PI_4".to_string(),
                (std::f64::consts::FRAC_PI_4.to_string(), TypeOfString::F64),
            ),
            (
                "FRAC_PI_6".to_string(),
                (std::f64::consts::FRAC_PI_6.to_string(), TypeOfString::F64),
            ),
            (
                "FRAC_PI_8".to_string(),
                (std::f64::consts::FRAC_PI_8.to_string(), TypeOfString::F64),
            ),
            (
                "LN_2".to_string(),
                (std::f64::consts::LN_2.to_string(), TypeOfString::F64),
            ),
            (
                "LN_10".to_string(),
                (std::f64::consts::LN_10.to_string(), TypeOfString::F64),
            ),
            (
                "LOG2_10".to_string(),
                (std::f64::consts::LOG2_10.to_string(), TypeOfString::F64),
            ),
            (
                "LOG2_E".to_string(),
                (std::f64::consts::LOG2_E.to_string(), TypeOfString::F64),
            ),
            (
                "LOG10_2".to_string(),
                (std::f64::consts::LOG10_2.to_string(), TypeOfString::F64),
            ),
            (
                "LOG10_E".to_string(),
                (std::f64::consts::LOG10_E.to_string(), TypeOfString::F64),
            ),
            (
                "PI".to_string(),
                (std::f64::consts::PI.to_string(), TypeOfString::F64),
            ),
            (
                "SQRT_2".to_string(),
                (std::f64::consts::SQRT_2.to_string(), TypeOfString::F64),
            ),
            (
                "TAU".to_string(),
                (std::f64::consts::TAU.to_string(), TypeOfString::F64),
            ),
        ];
    }

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
                    let v = value.get(0).unwrap();
                    let num: i64 = match v {
                        Value::Number(x) => {
                            if x.is_f64() {
                                x.as_f64().unwrap() as i64
                            } else {
                                x.as_i64().unwrap()
                            }
                        }
                        Value::Bool(x) => {
                            if *x {
                                1
                            } else {
                                0
                            }
                        }
                        Value::String(x) => atoi(x.to_string()),
                        _ => 0,
                    };
                    Ok(to_value(num))
                })
                .function("float", |value| {
                    if value.is_empty() {
                        return Ok(to_value(std::f64::NAN));
                    }
                    let v = value.get(0).unwrap();
                    let num: f64 = match v {
                        Value::Number(x) => x.as_f64().unwrap(),
                        Value::Bool(x) => {
                            if *x {
                                1.0
                            } else {
                                0.0
                            }
                        }
                        Value::String(x) => match x.parse::<f64>() {
                            Ok(x) => x,
                            _ => std::f64::NAN,
                        },
                        _ => std::f64::NAN,
                    };

                    Ok(to_value(num))
                })
                .function("bool", |value| {
                    if value.is_empty() {
                        return Ok(to_value(false));
                    }
                    let v = value.get(0).unwrap();
                    let result: bool = match v {
                        Value::Number(x) => x.as_f64().unwrap() != 0.0,
                        Value::Bool(x) => *x,
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
                    let v = value.get(0).unwrap();
                    let result: String = match v {
                        Value::Number(x) => {
                            if x.is_f64() {
                                x.as_f64().unwrap().to_string()
                            } else {
                                x.as_i64().unwrap().to_string()
                            }
                        }
                        Value::Bool(x) => x.to_string(),
                        Value::String(x) => x.to_string(),
                        Value::Array(x) => serde_json::to_string(x).unwrap(),
                        Value::Object(x) => serde_json::to_string(x).unwrap(),
                        _ => String::from("null"),
                    };
                    Ok(to_value(result))
                })
        }

        if config.include_maths {
            for (key, (str_value, type_of)) in math_consts() {
                if type_of == TypeOfString::INT64 {
                    result = result.value(key, str_value.parse::<i64>().unwrap())
                } else if type_of == TypeOfString::F64 {
                    result = result.value(key, str_value.parse::<f64>().unwrap())
                } else {
                    panic!("math constants should just be integers and floats; not {:?}", type_of);
                }
            }
        }

        if config.include_regex {
            result = result.function("is_match", |value| {
                if value.len() < 2 {
                    return Ok(to_value(false));
                }
                let v = value.get(0).unwrap();
                let pattern = value.get(1).unwrap().to_string();

                let value: String = match v {
                    Value::Number(x) => x.as_f64().unwrap().to_string(),
                    Value::Bool(x) => x.to_string(),
                    Value::String(x) => x.to_string(),
                    Value::Array(x) => serde_json::to_string(x).unwrap(),
                    Value::Object(x) => serde_json::to_string(x).unwrap(),
                    _ => String::from("null"),
                };

                let prog = Regex::new(&pattern).unwrap();
                let is_match = prog.is_match(&value);

                Ok(to_value(is_match))
            })
        }

        if config.include_datetime {
            result = result
                .function("day", |values| {
                    let current_time = eval_tz_parse_args(values, 1);
                    Ok(resolver::to_value(current_time.date().day()))
                })
                .function("month", |values| {
                    let current_time = eval_tz_parse_args(values, 1);
                    Ok(resolver::to_value(current_time.date().month()))
                })
                .function("year", |values| {
                    let current_time = eval_tz_parse_args(values, 1);
                    Ok(resolver::to_value(current_time.date().year()))
                })
                .function("weekday", |values| {
                    let current_time = eval_tz_parse_args(values, 1);
                    Ok(resolver::to_value(
                        current_time.date().weekday().number_from_monday(),
                    ))
                })
                .function("is_weekday", |values| {
                    let current_time = eval_tz_parse_args(values, 1);
                    let weekday = current_time.date().weekday().number_from_monday();
                    Ok(resolver::to_value(weekday < 6))
                })
                .function("is_weekend", |values| {
                    let current_time = eval_tz_parse_args(values, 1);
                    let weekday = current_time.date().weekday();
                    let weekends = [chrono::Weekday::Sat, chrono::Weekday::Sun];
                    Ok(resolver::to_value(weekends.contains(&weekday)))
                })
                .function("time", |extract| {
                    if extract.len() < 2 {
                        let t = now("_".to_owned());
                        return Ok(resolver::to_value(t.hour()));
                    }
                    let v: String = match extract.get(1).unwrap() {
                        resolver::Value::Number(x) => {
                            if x.is_f64() {
                                x.as_f64().unwrap().to_string()
                            } else {
                                x.as_i64().unwrap().to_string()
                            }
                        }
                        resolver::Value::Bool(x) => x.to_string(),
                        resolver::Value::String(x) => x.to_string(),
                        resolver::Value::Array(x) => serde_json::to_string(x).unwrap(),
                        resolver::Value::Object(x) => serde_json::to_string(x).unwrap(),
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
                    Ok(resolver::to_value(result))
                })
        }

        return result;

        // TODO: is_nan(n), is_min_int(n), is_int_max(n), includes(arr)
        // TODO: min(arr), max(arr), abs(n), pow(n, p), sum(arr), reverse(arr), sort(arr), unique(arr)
    }

    fn eval_tz_parse_args(
        arguments: Vec<resolver::Value>,
        min_args: usize,
    ) -> chrono::DateTime<chrono_tz::Tz> {
        let default_tz = "_".to_owned();
        if arguments.is_empty() || arguments.len() < min_args {
            log::warn!("No arguments");
            return now(default_tz);
        }

        let v: Option<String> = match arguments.get(0).unwrap() {
            resolver::Value::String(x) => Some(x.to_string()),
            _ => None,
        };

        if v.is_none() {
            log::warn!("Invalid Timezone");
            return now(default_tz);
        }

        return now(v.unwrap());
    }

    fn now(tz: String) -> chrono::DateTime<chrono_tz::Tz> {
        let utc = chrono::offset::Utc::now();
        let naive_dt = chrono::NaiveDate::from_ymd(utc.year(), utc.month(), utc.day()).and_hms(
            utc.hour(),
            utc.minute(),
            utc.second(),
        );

        str_to_tz(tz).from_local_datetime(&naive_dt).unwrap()
    }

    fn str_to_tz(timezone: String) -> chrono_tz::Tz {
        match timezone.as_str() {
            consts::tz::US_ALASKA => chrono_tz::US::Alaska,
            consts::tz::US_ALEUTIAN => chrono_tz::US::Aleutian,
            consts::tz::US_ARIZONA => chrono_tz::US::Arizona,
            consts::tz::US_CENTRAL => chrono_tz::US::Central,
            consts::tz::US_EASTINDIANA => chrono_tz::US::EastIndiana,
            consts::tz::US_EASTERN => chrono_tz::US::Eastern,
            consts::tz::US_HAWAII => chrono_tz::US::Hawaii,
            consts::tz::US_INDIANA_STARKE => chrono_tz::US::IndianaStarke,
            consts::tz::US_MICHIGAN => chrono_tz::US::Michigan,
            consts::tz::US_MOUNTAIN => chrono_tz::US::Mountain,
            consts::tz::US_PACIFIC => chrono_tz::US::Pacific,
            consts::tz::US_SAMOA => chrono_tz::US::Samoa,
            _ => {
                log::warn!("Defaulted to UTC timezone");
                return chrono_tz::UTC;
            }
        }
    }

    fn atoi(s: String) -> i64 {
        let mut item = s
            .trim()
            .split(char::is_whitespace)
            .next()
            .unwrap()
            .split(char::is_alphabetic)
            .next()
            .unwrap();

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
            Ok(v) => return v,
            Err(error) => match error.kind() {
                std::num::IntErrorKind::NegOverflow => return std::i64::MIN,
                std::num::IntErrorKind::PosOverflow => return std::i64::MAX,
                std::num::IntErrorKind::InvalidDigit => {
                    let result = item.parse::<f64>();
                    match result {
                        Ok(v) => return v.round() as i64,
                        _ => return 0,
                    };
                }
                _ => return 0,
            },
        }
    }
}



#[cfg(test)]
mod eval {
    use chrono::offset::Utc as Date;
    use chrono::{Datelike, Timelike};
    use resolver::to_value;

    use crate::eval_wrapper;

    struct Spec;

    impl Spec {
        pub fn default() -> Self {
            Spec {}
        } 
        
        pub fn eval<S: AsRef<str>>(&self, expression: S) -> resolver::Value {
            let expr = eval_wrapper::expr_wrapper(
                resolver::Expr::new(expression.as_ref().to_owned()),
                eval_wrapper::EvalConfig::default()
            );
            let result = expr.exec();

            if result.is_err() {
                panic!(
                    "Failed to parse expression: \"{}\" {:?}",
                    expression.as_ref().to_owned(),
                    result
                )
            }

            return result.unwrap();
 
        }
    }

    #[test]
    fn global_variables() {
        let user_spec = Spec::default();
        assert_eq!(user_spec.eval("MIN_INT"), to_value(std::i64::MIN));
        assert_eq!(user_spec.eval("MAX_INT"), to_value(std::i64::MAX));
        assert_eq!(user_spec.eval("MAX_FLOAT"), to_value(std::f64::MAX));
        assert_eq!(user_spec.eval("MIN_FLOAT"), to_value(std::f64::MIN));
        assert_eq!(user_spec.eval("NAN"), to_value(std::f64::NAN));
        assert_eq!(user_spec.eval("INFINITY"), to_value(std::f64::INFINITY));
        assert_eq!(
            user_spec.eval("NEG_INFINITY"),
            to_value(std::f64::NEG_INFINITY)
        );
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
    fn str() {
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
        assert_eq!(user_spec.eval("float('')"), to_value(std::f64::NAN));
        assert_eq!(
            user_spec.eval("float('not a num')"),
            to_value(std::f64::NAN)
        );
        assert_eq!(user_spec.eval("float(ctx)"), to_value(std::f64::NAN));
        assert_eq!(
            user_spec.eval("float(array(42, 42))"),
            to_value(std::f64::NAN)
        );
        assert_eq!(user_spec.eval("float(0..42)"), to_value(std::f64::NAN));
        assert_eq!(user_spec.eval("float(null)"), to_value(std::f64::NAN));
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

        assert_eq!(user_spec.eval("day()"), day);
        assert_eq!(user_spec.eval("day('_')"), day);
    }

    #[test]
    fn month() {
        let user_spec = Spec::default();
        let date = Date::now().date();
        let month = date.month();

        assert_eq!(user_spec.eval("month()"), month);
        assert_eq!(user_spec.eval("month('_')"), month);
    }

    #[test]
    fn year() {
        let user_spec = Spec::default();
        let date = Date::now().date();
        let year = date.year();
        assert_eq!(user_spec.eval("year()"), year);
        assert_eq!(user_spec.eval("year('_')"), year);
    }

    #[test]
    fn weekday() {
        let user_spec = Spec::default();
        let weekday_num = Date::now().weekday().number_from_monday();
        assert_eq!(user_spec.eval("weekday('_')"), weekday_num);
        assert_eq!(user_spec.eval("is_weekday('_')"), weekday_num < 6);

        assert_eq!(user_spec.eval("weekday()"), weekday_num);
        assert_eq!(user_spec.eval("is_weekday()"), weekday_num < 6);
    }

    #[test]
    fn time() {
        let user_spec = Spec::default();
        assert_eq!(user_spec.eval("time('_', 'h')"), Date::now().time().hour());
        assert_eq!(user_spec.eval("time('_', 'm')"), Date::now().time().minute());
        assert_eq!(user_spec.eval("time('_', 's')"), Date::now().time().second());

        assert_eq!(user_spec.eval("time('_', 'hour')"), Date::now().time().hour());
        assert_eq!(
            user_spec.eval("time('_', 'minute')"),
            Date::now().time().minute()
        );
        assert_eq!(
            user_spec.eval("time('_', 'second')"),
            Date::now().time().second()
        );

        assert_eq!(user_spec.eval("time('_', 'hours')"), Date::now().time().hour());
        assert_eq!(
            user_spec.eval("time('_', 'minutes')"),
            Date::now().time().minute()
        );
        assert_eq!(
            user_spec.eval("time('_', 'seconds')"),
            Date::now().time().second()
        );
    }
}

#[cfg(test)]
mod type_of_string {

    use crate::eval_wrapper::TypeOfString;

    #[test]
    fn value_check() {
        assert_eq!(TypeOfString::INT64.value(), "INTEGER");
        assert_eq!(TypeOfString::F64.value(), "FLOAT");
        assert_eq!(TypeOfString::BOOLEAN.value(), "BOOLEAN");
        assert_eq!(TypeOfString::STRING.value(), "STRING");
        assert_eq!(TypeOfString::ARRAY.value(), "ARRAY");
        assert_eq!(TypeOfString::OBJECT.value(), "OBJECT");
        assert_eq!(TypeOfString::NULL.value(), "NULL");
    }

    #[test]
    fn rev_value_check() {
        assert_eq!(TypeOfString::from_value("INTEGER"), TypeOfString::INT64);
        assert_eq!(TypeOfString::from_value("FLOAT"), TypeOfString::F64);
        assert_eq!(TypeOfString::from_value("BOOLEAN"), TypeOfString::BOOLEAN);
        assert_eq!(TypeOfString::from_value("STRING"), TypeOfString::STRING);
        assert_eq!(TypeOfString::from_value("ARRAY"), TypeOfString::ARRAY);
        assert_eq!(TypeOfString::from_value("OBJECT"), TypeOfString::OBJECT);
        assert_eq!(TypeOfString::from_value("NULL"), TypeOfString::NULL);
        assert_eq!(TypeOfString::from_value("_"), TypeOfString::NULL);
    }
}


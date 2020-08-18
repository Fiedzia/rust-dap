#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(dap);

#[cfg(test)]
mod tests {
    use crate::dap;

    #[test]
    fn parse_string_value() {
        let expr: String = dap::StringValueParser::new()
        .parse("'abc'")
        .unwrap();
        assert_eq!(expr, "abc");
    }

    #[test]
    fn parse_int_value() {
        let expr: u64 = dap::IntValueParser::new()
        .parse("0")
        .unwrap();
        assert_eq!(expr, 0);
    }

    #[test]
    fn parse_bool_value() {
        let expr: bool = dap::BoolValueParser::new()
        .parse("true")
        .unwrap();
        assert_eq!(expr, true);

        let expr2: bool = dap::BoolValueParser::new()
        .parse("false")
        .unwrap();
        assert_eq!(expr2, false);
    }

}

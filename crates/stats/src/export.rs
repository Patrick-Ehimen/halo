//! CSV export.

/// A CSV document from a header and rows, quoting fields that need it
/// (commas, quotes, newlines). Lines end with `\r\n`, as RFC 4180 asks.
pub fn csv(header: &[&str], rows: &[Vec<String>]) -> String {
    let mut out = String::new();
    let line = |fields: &mut dyn Iterator<Item = &str>, out: &mut String| {
        let quoted: Vec<String> = fields.map(field).collect();
        out.push_str(&quoted.join(","));
        out.push_str("\r\n");
    };
    line(&mut header.iter().copied(), &mut out);
    for row in rows {
        line(&mut row.iter().map(String::as_str), &mut out);
    }
    out
}

fn field(s: &str) -> String {
    // A leading = + - @ could run as a formula in a spreadsheet; prefix it.
    let s = if s.starts_with(['=', '+', '-', '@']) && s.parse::<f64>().is_err() {
        format!("'{s}")
    } else {
        s.to_string()
    };
    if s.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quotes_only_what_needs_it() {
        let out = csv(
            &["intention", "minutes"],
            &[
                vec!["Write tests".into(), "50".into()],
                vec!["Review, then \"merge\"".into(), "-5".into()],
                vec!["line\nbreak".into(), "0".into()],
            ],
        );
        assert_eq!(
            out,
            "intention,minutes\r\nWrite tests,50\r\n\"Review, then \"\"merge\"\"\",-5\r\n\"line\nbreak\",0\r\n"
        );
    }

    #[test]
    fn defuses_spreadsheet_formulas() {
        let out = csv(&["x"], &[vec!["=SUM(A1)".into()], vec!["@cmd".into()]]);
        assert_eq!(out, "x\r\n'=SUM(A1)\r\n'@cmd\r\n");
    }
}

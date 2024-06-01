use regex::Regex;

pub struct DurationParser {
    pattern: Regex,
}

impl DurationParser {
    pub fn new() -> Self {
        let pattern = "([0-9]+):([0-9]+)";
        let pattern =
            Regex::new(pattern).unwrap_or_else(|_| panic!("invalid pattern: '{pattern}'"));
        DurationParser { pattern }
    }

    pub fn parse_duration(&self, raw: &str) -> Option<(usize, usize)> {
        let caps: Vec<_> = self
            .pattern
            .captures_iter(raw)
            .map(|c| c.extract::<2>())
            .flat_map(|(_, hm)| hm)
            .map(|x| x.parse::<usize>())
            .map(|r| r.map_or(0, |v| v))
            .collect();
        let h = caps.first()?;
        let m = caps.get(1)?;
        Some((*h, *m))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_parser() {
        let tests = [
            ("0:00", Some((0, 0))),
            ("1:00", Some((1, 0))),
            ("0:30", Some((0, 30))),
            ("0:0", Some((0, 0))),
            ("1.23", None),
            ("0.00", None),
        ];
        let parser = DurationParser::new();

        for (input, expected) in tests {
            let actual = parser.parse_duration(input);
            assert_eq!(actual, expected);
        }
    }
}

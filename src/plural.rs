pub trait Pluralize {
    fn plural(&self, count: usize) -> String;
}

impl Pluralize for &'static str {
    fn plural(&self, count: usize) -> String {
        match count {
            1 => format!("{} {}", count, self),
            _ => format!("{} {}s", count, self),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_pluralize() {
        assert_eq!("1 package", "package".plural(1));
        assert_eq!("2 packages", "package".plural(2));
        assert_eq!("4 packages", "package".plural(4));
    }
}

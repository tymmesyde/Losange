pub trait Format {
    type Output;

    fn escape(&self) -> Self::Output;
    fn no_line_breaks(&self) -> Self::Output;
}

impl Format for String {
    type Output = String;

    fn escape(&self) -> String {
        xml::escape::escape_str_attribute(self).to_string()
    }

    fn no_line_breaks(&self) -> String {
        self.replace('\n', "")
    }
}

impl Format for Option<String> {
    type Output = Option<String>;

    fn escape(&self) -> Option<String> {
        self.to_owned().map(|s| s.escape())
    }

    fn no_line_breaks(&self) -> Option<String> {
        self.to_owned().map(|s| s.no_line_breaks())
    }
}

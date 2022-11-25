pub(crate) fn parse(input: String) -> Box<nixel::Expression> {
    nixel::parse(input).expression
}

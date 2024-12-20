#[derive(Debug)]
pub enum JsonEvent<'a> {
    Null,
    #[allow(dead_code)]
    Bool(bool),
    Number(f64),
    String(&'a str),
    StartObject,
    EndObject,
    StartArray,
    EndArray,
    Key(&'a str),
}

pub struct JsonStreamParser<F>
where
    F: FnMut(JsonEvent),
{
    callback: F,
    buffer: Vec<u8>,
    start_pos: usize,
    offset: usize,
    states: Vec<ParserState>,
}

#[derive(Debug)]
pub enum JsonStreamParseError {
    UnexpectedEndOfInput,
    #[allow(dead_code)]
    UnexpectedChar(char),
    #[allow(dead_code)]
    InvalidArray(String),
    #[allow(dead_code)]
    InvalidLiteral(String),
    InvalidBoolean,
    InvalidNumber,
}

#[derive(Debug, PartialEq)]
enum ParserState {
    StringVal,
    Array,
    Object(bool), // true - expects key
    ObjectKey,
    ObjectValue,
    ObjectColon,
    Num(bool), // true - is negative
    True,
    False,
    Null,
}

trait ParseStrSlice {
    fn parse_str(&self) -> &str;
}

impl ParseStrSlice for &[u8] {
    fn parse_str(&self) -> &str {
        // Using from_utf8_unchecked allows to improve a performance up to 30%
        unsafe { std::str::from_utf8_unchecked(self) } // .map_err(|_| JsonStreamParseError::InvalidString)
    }
}

impl<F> JsonStreamParser<F>
where
    F: FnMut(JsonEvent),
{
    pub fn new(callback: F) -> Self {
        let buffer = Vec::<u8>::with_capacity(1024 * 32); // 32kB
        let states = Vec::<ParserState>::with_capacity(16);
        Self {
            callback,
            buffer,
            start_pos: 0,
            offset: 0,
            states,
        }
    }

    fn append_buffer(&mut self, chunk: &[u8]) {
        // TODO
        // Try to chain buffers instead of copying
        // https://github.com/tokio-rs/bytes/blob/master/tests/test_chain.rs
        if self.buffer.len() + chunk.len() < self.buffer.capacity() {
            self.buffer.extend_from_slice(chunk);
        } else {
            let start_pos = self.start_pos;
            self.offset = self.buffer.len() - start_pos;
            self.buffer.drain(..start_pos);

            // TODO: use custom cache friendly reallocation algorithm?
            self.buffer.extend_from_slice(chunk);
            self.start_pos = 0
        }
    }

    pub fn parse(&mut self, chunk: &[u8]) -> Result<bool, JsonStreamParseError> {
        self.append_buffer(chunk);

        if self.states.is_empty() {
            self.skip_whitespace();
        }

        let mut complete = self.parse_value()?;
        while complete && !self.states.is_empty() {
            complete = self.parse_value()?;
        }

        Ok(complete)
    }

    fn parse_element(&mut self, is_key: bool) -> Result<bool, JsonStreamParseError> {
        match self.peek_char() {
            Some('n') => {
                self.states.push(ParserState::Null);
                self.parse_null()
            }
            Some('t') => {
                self.states.push(ParserState::True);
                self.parse_true()
            }
            Some('f') => {
                self.states.push(ParserState::False);
                self.parse_false()
            }
            Some('"') => {
                let new_state = if is_key {
                    ParserState::ObjectKey
                } else {
                    ParserState::StringVal
                };
                self.states.push(new_state);
                self.unsafe_consume_one_char();
                self.parse_string(is_key)
            }
            Some('[') => {
                self.states.push(ParserState::Array);
                self.unsafe_consume_one_char();
                (self.callback)(JsonEvent::StartArray);
                self.parse_array()
            }
            Some('{') => {
                self.states.push(ParserState::Object(true));
                self.unsafe_consume_one_char();
                (self.callback)(JsonEvent::StartObject);
                self.parse_object(true)
            }
            Some(c) if c.is_ascii_digit() || c == '-' => {
                let is_negative = c == '-';
                if is_negative {
                    self.unsafe_consume_one_char();
                }

                self.states.push(ParserState::Num(is_negative));
                self.parse_number(is_negative)
            }
            Some(c) => Err(JsonStreamParseError::UnexpectedChar(c)),
            None => {
                if self.offset < self.buffer.len() {
                    Err(JsonStreamParseError::UnexpectedEndOfInput)
                } else {
                    Ok(false)
                }
            }
        }
    }

    fn parse_value(&mut self) -> Result<bool, JsonStreamParseError> {
        let complete = match self.states.last() {
            None => self.parse_element(false),
            Some(ParserState::Array) => self.parse_array(),
            Some(&ParserState::Object(expects_key)) => self.parse_object(expects_key),
            Some(ParserState::ObjectKey) => {
                let complete = self.parse_string(true)?;

                if complete {
                    self.states.pop();
                    self.set_parsing_object_expects_key(false);
                    self.states.push(ParserState::ObjectKey); // will be removed
                }

                Ok(complete)
            }
            Some(ParserState::ObjectValue) => {
                self.skip_whitespace();
                self.parse_element(false)
            }
            Some(ParserState::ObjectColon) => {
                let result = self.consume_char(':')?;
                self.skip_whitespace();
                Ok(result)
            }
            Some(ParserState::StringVal) => self.parse_string(false),
            Some(ParserState::Num(negative)) => self.parse_number(*negative),
            Some(ParserState::True) => self.parse_true(),
            Some(ParserState::False) => self.parse_false(),
            Some(ParserState::Null) => self.parse_null(),
        }?;
        if complete {
            self.states.pop();

            if let Some(top) = self.states.last() {
                if top == &ParserState::ObjectValue {
                    self.states.pop(); // remove ParsingObjectValue
                    self.set_parsing_object_expects_key(true); // expects key
                }
            }
        }
        Ok(complete)
    }

    #[inline(always)]
    fn set_parsing_object_expects_key(&mut self, value: bool) {
        let index = self.states.len() - 1;
        let ParserState::Object(expects_key) =
            (unsafe { &mut self.states.get_unchecked_mut(index) })
        else {
            unreachable!()
        };
        *expects_key = value;
    }

    #[inline(always)]
    fn parse_null(&mut self) -> Result<bool, JsonStreamParseError> {
        let complete = self.expect_literal(b"null")?;
        if complete {
            (self.callback)(JsonEvent::Null);
            self.start_pos = self.offset;
        }
        Ok(complete)
    }

    #[inline(always)]
    fn parse_true(&mut self) -> Result<bool, JsonStreamParseError> {
        let complete = self
            .expect_literal(b"true")
            .map_err(|_| JsonStreamParseError::InvalidBoolean)?;
        if complete {
            (self.callback)(JsonEvent::Bool(true));
            self.start_pos = self.offset;
        }
        Ok(complete)
    }

    #[inline(always)]
    fn parse_false(&mut self) -> Result<bool, JsonStreamParseError> {
        let complete = self
            .expect_literal(b"false")
            .map_err(|_| JsonStreamParseError::InvalidBoolean)?;
        if complete {
            (self.callback)(JsonEvent::Bool(false));
            self.start_pos = self.offset;
        }
        Ok(complete)
    }

    fn parse_number(&mut self, is_negative: bool) -> Result<bool, JsonStreamParseError> {
        let is_complete;

        loop {
            if let Some(chr) = self.peek_char() {
                if chr.is_ascii_digit() || chr == '.' || chr == '-' || chr == 'e' || chr == 'E' {
                    self.next_char();
                } else if self.states.len() == 1 {
                    // num is a top level element
                    return Err(JsonStreamParseError::InvalidNumber);
                } else {
                    is_complete = true;
                    break;
                }
            } else {
                is_complete = self.states.len() == 1; // is top level
                break;
            }
        }

        if is_complete {
            let bytes = &self.buffer[self.start_pos..self.offset];
            let slice = bytes.parse_str();

            // TODO maybe will be efficient to parse into int if no dots in string
            // here used: https://github.com/rust-lang/rust/blob/master/library/core/src/num/dec2flt/mod.rs
            return match slice.parse::<f64>() {
                Ok(n) => {
                    let num = if is_negative { -n } else { n };
                    (self.callback)(JsonEvent::Number(num));
                    self.start_pos = self.offset;
                    Ok(true)
                }
                Err(_) => Err(JsonStreamParseError::InvalidNumber),
            };
        }

        Ok(false)
    }

    // TODO
    // 1. handle escaped characters
    // 2. search with simd can be more efficient
    fn parse_string(&mut self, is_key: bool) -> Result<bool, JsonStreamParseError> {
        while let Some(c) = self.next_char() {
            if c == '"' {
                let bytes = &self.buffer[self.start_pos..(self.offset - 1)];
                let slice = bytes.parse_str();
                let value = if is_key {
                    JsonEvent::Key(slice)
                } else {
                    JsonEvent::String(slice)
                };
                (self.callback)(value);
                self.start_pos = self.offset;

                return Ok(true);
            }
        }

        Ok(false)
    }

    fn parse_array(&mut self) -> Result<bool, JsonStreamParseError> {
        loop {
            self.skip_whitespace();
            if self.peek_char() == Some(']') {
                self.unsafe_consume_one_char();
                (self.callback)(JsonEvent::EndArray);
                return Ok(true);
            }

            if let Some(last_char) = self.peek_char() {
                // hack, use separate parsing states
                if last_char == ',' {
                    self.unsafe_consume_one_char();
                    self.start_pos = self.offset;
                    self.skip_whitespace();
                }

                let complete = self.parse_element(false)?;
                if complete {
                    self.states.pop();
                    self.skip_whitespace();

                    if let Some(last_char) = self.peek_char() {
                        if last_char != ']' && last_char != ',' {
                            return Err(JsonStreamParseError::InvalidArray(
                                "Expected ',' or ']'".into(),
                            ));
                        }
                        continue;
                    }
                }
            }
            break;
        }

        Ok(false)
    }

    #[allow(dead_code)]
    #[inline(always)]
    fn print_rem(&self) {
        let bytes = &self.buffer[self.start_pos..];
        let slice = bytes.parse_str(); // .expect("unsafe debug method");
        println!("rem: {}, pos: {}", slice, self.start_pos);
    }

    fn parse_object(&mut self, expects_key: bool) -> Result<bool, JsonStreamParseError> {
        let mut expects_key = expects_key;

        loop {
            self.skip_whitespace();
            if self.peek_char() == Some('}') {
                self.unsafe_consume_one_char();
                (self.callback)(JsonEvent::EndObject);
                return Ok(true);
            }

            if let Some(last_char) = self.peek_char() {
                // hack, use separate parsing states
                if last_char == ':' || last_char == ',' {
                    self.unsafe_consume_one_char();
                    self.start_pos = self.offset;
                    self.skip_whitespace();
                }

                if !expects_key {
                    self.states.push(ParserState::ObjectValue);
                }

                let complete = self.parse_element(expects_key)?; // Parse key or value
                if complete {
                    self.states.pop();
                    if !expects_key {
                        self.states.pop(); // pop ParsingObjectValue
                    }
                    expects_key = !expects_key;
                    self.set_parsing_object_expects_key(expects_key);

                    if !expects_key {
                        self.skip_whitespace();
                        self.states.push(ParserState::ObjectColon);
                        let complete = self.consume_char(':')?;
                        if complete {
                            self.states.pop();
                            self.skip_whitespace();
                        } else {
                            break;
                        }
                    }
                    continue;
                }
            }
            break;
        }

        Ok(false)
    }

    #[allow(dead_code)]
    #[inline(always)]
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.next_char();
                self.start_pos += 1;
            } else {
                break;
            }
        }
    }

    // TODO compare with simd can be more efficient
    #[inline(always)]
    fn expect_literal(&mut self, literal: &[u8]) -> Result<bool, JsonStreamParseError> {
        let start_pos = self.offset - self.start_pos;
        for expected in &literal[start_pos..] {
            if let Some(next_char) = self.next_byte() {
                if next_char != *expected {
                    return Err(JsonStreamParseError::InvalidLiteral(
                        "Unexpected literal".into(),
                    ));
                }
            } else {
                return Ok(false);
            }
        }
        Ok(true)
    }

    #[inline(always)]
    fn consume_char(&mut self, expected: char) -> Result<bool, JsonStreamParseError> {
        if let Some(next_char) = self.next_char() {
            if next_char == expected {
                self.start_pos += 1;
                Ok(true)
            } else {
                Err(JsonStreamParseError::InvalidLiteral(format!(
                    "Expected '{}'",
                    expected
                )))
            }
        } else {
            Ok(false)
        }
    }

    #[inline(always)]
    fn unsafe_consume_one_char(&mut self) {
        self.offset += 1;
        self.start_pos += 1
    }

    #[inline(always)]
    fn peek_char(&self) -> Option<char> {
        if self.offset < self.buffer.len() {
            Some(unsafe { *self.buffer.get_unchecked(self.offset) } as char)
        } else {
            None
        }
    }

    #[inline(always)]
    fn next_byte(&mut self) -> Option<u8> {
        if self.offset < self.buffer.len() {
            let result = unsafe { *self.buffer.get_unchecked(self.offset) };
            self.offset += 1;
            Some(result)
        } else {
            None
        }
    }

    #[inline(always)]
    fn next_char(&mut self) -> Option<char> {
        self.next_byte().map(|x| x as char)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[derive(Debug, PartialEq, Clone)]
    pub enum OwningJsonEvent {
        Null,
        Bool(bool),
        Number(f64),
        String(String),
        StartObject,
        EndObject,
        StartArray,
        EndArray,
        Key(String),
    }

    impl<'a> From<JsonEvent<'a>> for OwningJsonEvent {
        fn from(value: JsonEvent<'a>) -> Self {
            match value {
                JsonEvent::Null => OwningJsonEvent::Null,
                JsonEvent::Bool(bool) => OwningJsonEvent::Bool(bool),
                JsonEvent::Number(number) => OwningJsonEvent::Number(number),
                JsonEvent::String(string) => OwningJsonEvent::String(string.to_string()),
                JsonEvent::StartObject => OwningJsonEvent::StartObject,
                JsonEvent::EndObject => OwningJsonEvent::EndObject,
                JsonEvent::StartArray => OwningJsonEvent::StartArray,
                JsonEvent::EndArray => OwningJsonEvent::EndArray,
                JsonEvent::Key(key) => OwningJsonEvent::Key(key.to_string()),
            }
        }
    }

    fn test_json_parser_element(element: &str, expected: OwningJsonEvent, chunked: bool) {
        let json = element;

        let events: RefCell<Vec<OwningJsonEvent>> = RefCell::new(vec![]);

        let bytes = json.as_bytes();

        let index = RefCell::new(0);
        let get_next_idx = || {
            let prev_index = *index.borrow();
            *index.borrow_mut() = prev_index + 1;
            prev_index
        };

        fn test(
            events: &[OwningJsonEvent],
            expected: OwningJsonEvent,
            mut inc: impl FnMut() -> usize,
        ) {
            assert_eq!(&events[inc()], &expected);
        }

        if chunked {
            for split_at in 1..json.len() {
                events.borrow_mut().clear();
                // Debug: println!("testing split at: {split_at}, [{}][{}]", &json[0..split_at], &json[split_at..]);
                let mut parser =
                    JsonStreamParser::new(|event| events.borrow_mut().push(event.into()));

                assert!(parser.parse(&bytes[0..split_at]).is_ok());
                assert!(parser.parse(&bytes[split_at..]).is_ok());
                *index.borrow_mut() = 0;
                test(&events.borrow(), expected.clone(), get_next_idx);
            }
        }

        events.borrow_mut().clear();

        let mut parser = JsonStreamParser::new(|event| events.borrow_mut().push(event.into()));
        assert!(parser.parse(bytes).is_ok());

        *index.borrow_mut() = 0;
        test(&events.borrow(), expected.clone(), get_next_idx);
    }

    #[test]
    fn test_json_parser_null() {
        let literal = "null";
        let expected = OwningJsonEvent::Null;
        test_json_parser_element(literal, expected, true);
    }

    #[test]
    fn test_json_parser_true() {
        let literal = "true";
        let expected = OwningJsonEvent::Bool(true);
        test_json_parser_element(literal, expected, true);
    }

    #[test]
    fn test_json_parser_false() {
        let literal = "false";
        let expected = OwningJsonEvent::Bool(false);
        test_json_parser_element(literal, expected, true);
    }

    #[test]
    fn test_json_parser_string() {
        let literal = "\"test string\"";
        let expected = OwningJsonEvent::String("test string".into());
        test_json_parser_element(literal, expected, true);
    }

    #[test]
    fn test_json_parser_number() {
        let literal = "56";
        let expected = OwningJsonEvent::Number(56.0);
        test_json_parser_element(literal, expected, false);
    }

    #[test]
    fn test_json_parser_array() {
        let json = r#"
            [ 56.3, "Rust" , true, false , null ]
        "#;

        let events: RefCell<Vec<OwningJsonEvent>> = RefCell::new(vec![]);

        let bytes = json.as_bytes();

        let index = RefCell::new(0);
        let get_next_idx = || {
            let prev_index = *index.borrow();
            *index.borrow_mut() = prev_index + 1;
            prev_index
        };

        fn test(events: &[OwningJsonEvent], mut inc: impl FnMut() -> usize) {
            assert_eq!(&events[inc()], &OwningJsonEvent::StartArray);
            assert_eq!(&events[inc()], &OwningJsonEvent::Number(56.3));
            assert_eq!(&events[inc()], &OwningJsonEvent::String("Rust".into()));
            assert_eq!(&events[inc()], &OwningJsonEvent::Bool(true));
            assert_eq!(&events[inc()], &OwningJsonEvent::Bool(false));
            assert_eq!(&events[inc()], &OwningJsonEvent::Null);
            assert_eq!(&events[inc()], &OwningJsonEvent::EndArray);
        }

        for split_at in 1..json.len() {
            events.borrow_mut().clear();
            // Debug: println!("testing split at: {split_at}, '{}'+'{}'", &json[0..split_at], &json[split_at..]);
            let mut parser = JsonStreamParser::new(|event| events.borrow_mut().push(event.into()));

            assert!(parser.parse(&bytes[0..split_at]).is_ok());
            assert!(parser.parse(&bytes[split_at..]).is_ok());
            *index.borrow_mut() = 0;
            test(&events.borrow(), get_next_idx);
        }

        events.borrow_mut().clear();

        let mut parser = JsonStreamParser::new(|event| events.borrow_mut().push(event.into()));
        assert!(parser.parse(bytes).is_ok());

        *index.borrow_mut() = 0;
        test(&events.borrow(), get_next_idx);
    }

    // TODO test parse empty array

    #[test]
    fn test_json_parser_object() {
        let json = r#"
            {
                "name": "Alice",
                "age": 30,
                "is_active": true,
                "married": false,
                "skills": ["Rust", "C++"]
            }
        "#;

        let events: RefCell<Vec<OwningJsonEvent>> = RefCell::new(vec![]);

        let bytes = json.as_bytes();

        let index = RefCell::new(0);
        let get_next_idx = || {
            let prev_index = *index.borrow();
            *index.borrow_mut() = prev_index + 1;
            prev_index
        };

        fn test(events: &[OwningJsonEvent], mut inc: impl FnMut() -> usize) {
            assert_eq!(&events[inc()], &OwningJsonEvent::StartObject);

            assert_eq!(&events[inc()], &OwningJsonEvent::Key("name".to_string()));
            assert_eq!(
                &events[inc()],
                &OwningJsonEvent::String("Alice".to_string())
            );

            assert_eq!(&events[inc()], &OwningJsonEvent::Key("age".to_string()));
            assert_eq!(&events[inc()], &OwningJsonEvent::Number(30.0));

            assert_eq!(
                &events[inc()],
                &OwningJsonEvent::Key("is_active".to_string())
            );
            assert_eq!(&events[inc()], &OwningJsonEvent::Bool(true));

            assert_eq!(&events[inc()], &OwningJsonEvent::Key("married".to_string()));
            assert_eq!(&events[inc()], &OwningJsonEvent::Bool(false));

            assert_eq!(&events[inc()], &OwningJsonEvent::Key("skills".to_string()));

            assert_eq!(&events[inc()], &OwningJsonEvent::StartArray);
            {
                assert_eq!(&events[inc()], &OwningJsonEvent::String("Rust".to_string()));
                assert_eq!(&events[inc()], &OwningJsonEvent::String("C++".to_string()));
            }
            assert_eq!(&events[inc()], &OwningJsonEvent::EndArray);

            assert_eq!(&events[inc()], &OwningJsonEvent::EndObject);
        }

        for split_at in 1..json.len() {
            events.borrow_mut().clear();
            // Debug: println!("testing split at: {split_at}, '{}'+'{}'", &json[0..split_at], &json[split_at..]);
            let mut parser = JsonStreamParser::new(|event| {
                events.borrow_mut().push(event.into());
            });

            assert!(parser.parse(&bytes[0..split_at]).is_ok());
            assert!(parser.parse(&bytes[split_at..]).is_ok());
            *index.borrow_mut() = 0;
            test(&events.borrow(), get_next_idx);
        }

        events.borrow_mut().clear();

        let mut parser = JsonStreamParser::new(|event| events.borrow_mut().push(event.into()));
        assert!(parser.parse(bytes).is_ok());

        *index.borrow_mut() = 0;
        test(&events.borrow(), get_next_idx);
    }

    #[test]
    fn test_json_parser_array_of_object_with_array() {
        let json = r#"
            [
                {
                    "name": "Alice",
                    "age": 30,
                    "is_active": true,
                    "married": false,
                    "skills": ["Rust", "C++"]
                },
                {
                    "name": "Vova",
                    "age": 40,
                    "is_active": false,
                    "married": true,
                    "skills": ["Rust", "C++", "Scala"]
                }
            ]
        "#;

        let events: RefCell<Vec<OwningJsonEvent>> = RefCell::new(vec![]);

        let bytes = json.as_bytes();

        let index = RefCell::new(0);
        let get_next_idx = || {
            let prev_index = *index.borrow();
            *index.borrow_mut() = prev_index + 1;
            prev_index
        };

        fn test(events: &[OwningJsonEvent], mut inc: impl FnMut() -> usize) {
            assert_eq!(&events[inc()], &OwningJsonEvent::StartArray);

            // Alice
            {
                assert_eq!(&events[inc()], &OwningJsonEvent::StartObject);

                assert_eq!(&events[inc()], &OwningJsonEvent::Key("name".to_string()));
                assert_eq!(
                    &events[inc()],
                    &OwningJsonEvent::String("Alice".to_string())
                );

                assert_eq!(&events[inc()], &OwningJsonEvent::Key("age".to_string()));
                assert_eq!(&events[inc()], &OwningJsonEvent::Number(30.0));

                assert_eq!(
                    &events[inc()],
                    &OwningJsonEvent::Key("is_active".to_string())
                );
                assert_eq!(&events[inc()], &OwningJsonEvent::Bool(true));

                assert_eq!(&events[inc()], &OwningJsonEvent::Key("married".to_string()));
                assert_eq!(&events[inc()], &OwningJsonEvent::Bool(false));

                assert_eq!(&events[inc()], &OwningJsonEvent::Key("skills".to_string()));

                assert_eq!(&events[inc()], &OwningJsonEvent::StartArray);
                {
                    assert_eq!(&events[inc()], &OwningJsonEvent::String("Rust".to_string()));
                    assert_eq!(&events[inc()], &OwningJsonEvent::String("C++".to_string()));
                }
                assert_eq!(&events[inc()], &OwningJsonEvent::EndArray);

                assert_eq!(&events[inc()], &OwningJsonEvent::EndObject);
            }

            // Vova
            {
                assert_eq!(&events[inc()], &OwningJsonEvent::StartObject);

                assert_eq!(&events[inc()], &OwningJsonEvent::Key("name".to_string()));
                assert_eq!(&events[inc()], &OwningJsonEvent::String("Vova".to_string()));

                assert_eq!(&events[inc()], &OwningJsonEvent::Key("age".to_string()));
                assert_eq!(&events[inc()], &OwningJsonEvent::Number(40.0));

                assert_eq!(
                    &events[inc()],
                    &OwningJsonEvent::Key("is_active".to_string())
                );
                assert_eq!(&events[inc()], &OwningJsonEvent::Bool(false));

                assert_eq!(&events[inc()], &OwningJsonEvent::Key("married".to_string()));
                assert_eq!(&events[inc()], &OwningJsonEvent::Bool(true));

                assert_eq!(&events[inc()], &OwningJsonEvent::Key("skills".to_string()));

                assert_eq!(&events[inc()], &OwningJsonEvent::StartArray);
                {
                    assert_eq!(&events[inc()], &OwningJsonEvent::String("Rust".to_string()));
                    assert_eq!(&events[inc()], &OwningJsonEvent::String("C++".to_string()));
                    assert_eq!(
                        &events[inc()],
                        &OwningJsonEvent::String("Scala".to_string())
                    );
                }
                assert_eq!(&events[inc()], &OwningJsonEvent::EndArray);

                assert_eq!(&events[inc()], &OwningJsonEvent::EndObject);
            }

            assert_eq!(&events[inc()], &OwningJsonEvent::EndArray);
        }

        for split_at in 1..json.len() {
            events.borrow_mut().clear();
            // Debug: println!("testing split at: {split_at}, '{}'+'{}'", &json[0..split_at], &json[split_at..]);
            let mut parser = JsonStreamParser::new(|event| {
                events.borrow_mut().push(event.into());
            });

            assert!(parser.parse(&bytes[0..split_at]).is_ok());
            assert!(parser.parse(&bytes[split_at..]).is_ok());
            *index.borrow_mut() = 0;
            test(&events.borrow(), get_next_idx);
        }

        events.borrow_mut().clear();

        let mut parser = JsonStreamParser::new(|event| events.borrow_mut().push(event.into()));
        assert!(parser.parse(bytes).is_ok());

        *index.borrow_mut() = 0;
        test(&events.borrow(), get_next_idx);
    }
}

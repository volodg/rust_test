#[derive(Debug)]
pub enum JsonEvent<'a> {
    Null,
    #[allow(dead_code)]
    Bool(bool),
    #[allow(dead_code)]
    Number(f64),
    #[allow(dead_code)]
    String(&'a str),
    StartObject,
    EndObject,
    StartArray,
    EndArray,
    #[allow(dead_code)]
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

// TODO:
// 1. measure performance
// 2. create producer/consumer
// 3. Generate test data

#[derive(Debug)]
pub enum JsonStreamParseError {
    UnexpectedEndOfInput,
    #[allow(dead_code)]
    UnexpectedChar(char),
    #[allow(dead_code)]
    InvalidArray(String),
    #[allow(dead_code)]
    InvalidObject(String),
    #[allow(dead_code)]
    InvalidLiteral(String),
    InvalidBoolean,
    InvalidNumber,
}

#[derive(Debug, PartialEq)]
enum ParserState {
    ParsingKey,
    ParsingString,
    ParsingArray,
    ParsingObject,
    ParsingNum(bool), // true - is negative
    ParsingTrue,
    ParsingFalse,
    ParsingNull,
    ParsingObjColon,
}

impl<'a, F> JsonStreamParser<F>
where
    F: FnMut(JsonEvent),
{
    pub fn new(callback: F) -> Self {
        // TODO: play with initial size
        let buffer = Vec::<u8>::with_capacity(1024); // 1k
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
        let start_pos = self.start_pos;
        self.offset = self.buffer.len() - start_pos;
        self.buffer.drain(..start_pos);

        // TODO: use custom cache friendly reallocation algorithm?
        self.buffer.extend_from_slice(chunk);
        self.start_pos = 0
    }

    pub fn parse(&mut self, chunk: &[u8]) -> Result<bool, JsonStreamParseError> {
        self.append_buffer(chunk);

        if self.states.is_empty() {
            self.skip_whitespace();
        }

        let mut complete = self.parse_value()?;
        while complete {
            if let Some(top) = self.states.last() {
                if top == &ParserState::ParsingArray {
                    self.skip_whitespace();

                    if let Some(last_char) = self.peek_char() {
                        if last_char == ',' {
                            self.unsafe_consume_one_char();
                            self.start_pos = self.offset;
                            self.skip_whitespace();
                        } else if last_char == ']' {
                            self.unsafe_consume_one_char();
                            self.skip_whitespace();
                            self.states.pop(); // pop ParsingArray
                            (self.callback)(JsonEvent::EndArray);
                            continue;
                        } else {
                            return Err(JsonStreamParseError::InvalidArray("Expected ',' or ']'".into()));
                        }
                        complete = self.parse_value()?;
                    } else {
                        return Ok(false);
                    }
                }
            } else {
                return Ok(true);
            }
        }

        Ok(complete)
    }

    fn parse_element(&mut self, is_key: bool) -> Result<bool, JsonStreamParseError> {
        match self.peek_char() {
            Some('n') => {
                self.states.push(ParserState::ParsingNull);
                self.parse_null()
            }
            Some('t') => {
                self.states.push(ParserState::ParsingTrue);
                self.parse_true()
            }
            Some('f') => {
                self.states.push(ParserState::ParsingFalse);
                self.parse_false()
            }
            Some('"') => {
                let new_state = if is_key {
                    ParserState::ParsingKey
                } else {
                    ParserState::ParsingString
                };
                self.states.push(new_state);
                self.unsafe_consume_one_char();
                self.parse_string(is_key)
            }
            Some('[') => {
                self.states.push(ParserState::ParsingArray);
                self.unsafe_consume_one_char();
                (self.callback)(JsonEvent::StartArray);
                self.parse_array()
            }
            Some('{') => {
                self.states.push(ParserState::ParsingObject);
                self.unsafe_consume_one_char();
                (self.callback)(JsonEvent::StartObject);
                self.parse_object()
            }
            Some(c) if c.is_digit(10) || c == '-' => {
                let is_negative = c == '-';
                if is_negative {
                    self.unsafe_consume_one_char();
                }

                self.states.push(ParserState::ParsingNum(is_negative));
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
            None => {
                self.parse_element(false)
            }
            Some(ParserState::ParsingArray) => {
                // TODO fix code duplications
                self.skip_whitespace();
                if let Some(last_char) = self.peek_char() {
                    if last_char == ',' {
                        self.unsafe_consume_one_char();
                        self.start_pos = self.offset;
                        self.skip_whitespace();
                        self.parse_element(false)
                    } else if last_char == ']' {
                        self.unsafe_consume_one_char();
                        self.skip_whitespace();
                        (self.callback)(JsonEvent::EndArray);
                        Ok(true)
                    } else {
                        self.parse_element(false)
                    }
                } else {
                    Ok(false)
                }
            }
            Some(ParserState::ParsingObject) => {
                todo!()
            }
            Some(ParserState::ParsingKey) => {
                self.parse_string(true)
            }
            Some(ParserState::ParsingObjColon) => {
                let result = self.consume_char(':')?;
                self.skip_whitespace();
                Ok(result)
            }
            Some(ParserState::ParsingString) => {
                self.parse_string(false)
            }
            Some(ParserState::ParsingNum(negative)) => {
                self.parse_number(*negative)
            }
            Some(ParserState::ParsingTrue) => {
                self.parse_true()
            }
            Some(ParserState::ParsingFalse) => {
                self.parse_false()
            }
            Some(ParserState::ParsingNull) => {
                self.parse_null()
            }
        }?;
        if complete {
            self.states.pop();
        }
        Ok(complete)
    }

    fn parse_null(&mut self) -> Result<bool, JsonStreamParseError> {
        let complete = self.expect_literal("null")?;
        if complete {
            (self.callback)(JsonEvent::Null);
            self.start_pos = self.offset;
        }
        Ok(complete)
    }

    fn parse_true(&mut self) -> Result<bool, JsonStreamParseError> {
        let complete = self.expect_literal("true").map_err(|_| JsonStreamParseError::InvalidBoolean)?;
        if complete {
            (self.callback)(JsonEvent::Bool(true));
            self.start_pos = self.offset;
        }
        Ok(complete)
    }

    fn parse_false(&mut self) -> Result<bool, JsonStreamParseError> {
        let complete = self.expect_literal("false").map_err(|_| JsonStreamParseError::InvalidBoolean)?;
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
                if chr.is_digit(10) || chr == '.' || chr == '-' || chr == 'e' || chr == 'E' {
                    self.next_char();
                } else {
                    if self.states.len() == 1 { // num is a top level element
                        return Err(JsonStreamParseError::InvalidNumber);
                    } else {
                        is_complete = true;
                        break;
                    }
                }
            } else {
                is_complete = self.states.len() == 1; // is top level
                break;
            }
        }

        if is_complete {
            let bytes = &self.buffer[self.start_pos..self.offset];
            let slice = std::str::from_utf8(bytes).expect(""); // TODO avoid expects

            return match slice.parse::<f64>() {
                Ok(n) => {
                    if is_negative {
                        (self.callback)(JsonEvent::Number(-n));
                    } else {
                        (self.callback)(JsonEvent::Number(n));
                    }
                    self.start_pos = self.offset;
                    Ok(true)
                }
                Err(_) => Err(JsonStreamParseError::InvalidNumber),
            };
        }

        Ok(false)
    }

    // TODO handle escaped characters
    fn parse_string(&mut self, is_key: bool) -> Result<bool, JsonStreamParseError> {
        while let Some(c) = self.next_char() {
            if c == '"' {
                let bytes = &self.buffer[self.start_pos..(self.offset - 1)];
                let slice = std::str::from_utf8(bytes).expect(""); // TODO avoid expects
                if is_key {
                    (self.callback)(JsonEvent::Key(slice));
                } else {
                    (self.callback)(JsonEvent::String(slice));
                }
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

            let complete = self.parse_element(false)?;
            if complete {
                self.states.pop();
                self.skip_whitespace();

                // TODO fix code duplication
                // TODO existing char is expected
                if let Some(last_char) = self.peek_char() {
                    if last_char == ',' {
                        self.unsafe_consume_one_char();
                        self.start_pos = self.offset;
                        self.skip_whitespace();
                    } else if last_char != ']' {
                        return Err(JsonStreamParseError::InvalidArray("Expected ',' or ']'".into()));
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        Ok(false)
    }

    #[allow(dead_code)]
    fn print_rem(&self) {
        let bytes = &self.buffer[self.start_pos..];
        let slice = std::str::from_utf8(bytes).expect(""); // TODO avoid expects
        println!("rem: {}, pos: {}", slice, self.start_pos);
    }

    // TODO review
    fn parse_object(&mut self) -> Result<bool, JsonStreamParseError> {
        loop {
            self.skip_whitespace();
            if self.peek_char() == Some('}') {
                self.unsafe_consume_one_char();
                (self.callback)(JsonEvent::EndObject);
                return Ok(true);
            }

            let complete = self.parse_element(true)?; // Parse key
            if complete {
                self.states.pop();

                self.skip_whitespace();
                self.states.push(ParserState::ParsingObjColon);
                self.consume_char(':')?;
                self.states.pop();
                self.skip_whitespace();

                let complete = self.parse_element(false)?;

                if complete {
                    self.states.pop();
                    self.skip_whitespace();

                    if self.peek_char() == Some(',') {
                        self.unsafe_consume_one_char();
                    } else if self.peek_char() != Some('}') {
                        return Err(JsonStreamParseError::InvalidObject("Expected ',' or '}'".into()));
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        Ok(false)
    }

    #[allow(dead_code)]
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

    // TODO use &[u8] with O(1) indexing for literal
    fn expect_literal(&mut self, literal: &str) -> Result<bool, JsonStreamParseError> {
        let start_pos = self.offset - self.start_pos;
        for expected in literal[start_pos..].chars() {
            if let Some(next_char) = self.next_char() {
                if next_char != expected {
                    return Err(JsonStreamParseError::InvalidLiteral(format!("Expected literal: {}", literal)));
                }
            } else {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn consume_char(&mut self, expected: char) -> Result<bool, JsonStreamParseError> {
        if self.next_char() == Some(expected) {
            self.start_pos += 1;
            Ok(true)
        } else {
            return Err(JsonStreamParseError::InvalidLiteral(format!("Expected '{}'", expected)));
        }
    }

    fn unsafe_consume_one_char(&mut self) {
        self.offset += 1;
        self.start_pos += 1
    }

    fn peek_char(&self) -> Option<char> {
        if self.offset < self.buffer.len() {
            Some(self.buffer[self.offset] as char)
        } else {
            None
        }
    }

    fn next_char(&mut self) -> Option<char> {
        if self.offset < self.buffer.len() {
            let result = self.buffer[self.offset] as char;
            self.offset += 1;
            Some(result)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use super::*;

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

        fn test(events: &[OwningJsonEvent], expected: OwningJsonEvent, mut inc: impl FnMut() -> usize) {
            assert_eq!(&events[inc()], &expected);
        }

        if chunked {
            for split_at in 1..json.len() {
                events.borrow_mut().clear();
                println!("testing split at: {split_at}, [{}][{}]", &json[0..split_at], &json[split_at..]);
                let mut parser = JsonStreamParser::new(|event| {
                    events.borrow_mut().push(event.into())
                });

                assert!(parser.parse(&bytes[0..split_at]).is_ok());
                assert!(parser.parse(&bytes[split_at..]).is_ok());
                *index.borrow_mut() = 0;
                test(&events.borrow(), expected.clone(), get_next_idx);
            }
        }

        events.borrow_mut().clear();

        let mut parser = JsonStreamParser::new(|event| {
            events.borrow_mut().push(event.into())
        });
        assert!(parser.parse(&bytes).is_ok());

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
            println!("testing split at: {split_at}, '{}'+'{}'", &json[0..split_at], &json[split_at..]);
            let mut parser = JsonStreamParser::new(|event| {
                events.borrow_mut().push(event.into())
            });

            assert!(parser.parse(&bytes[0..split_at]).is_ok());
            assert!(parser.parse(&bytes[split_at..]).is_ok());
            *index.borrow_mut() = 0;
            test(&events.borrow(), get_next_idx);
        }

        events.borrow_mut().clear();

        let mut parser = JsonStreamParser::new(|event| {
            events.borrow_mut().push(event.into())
        });
        assert!(parser.parse(&bytes).is_ok());

        *index.borrow_mut() = 0;
        test(&events.borrow(), get_next_idx);
    }

    // TODO test parse empty array

    // TODO complete
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
            assert_eq!(&events[inc()], &OwningJsonEvent::String("Alice".to_string()));

            assert_eq!(&events[inc()], &OwningJsonEvent::Key("age".to_string()));
            assert_eq!(&events[inc()], &OwningJsonEvent::Number(30.0));

            assert_eq!(&events[inc()], &OwningJsonEvent::Key("is_active".to_string()));
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

        // for split_at in 1..json.len() {
        //     events.borrow_mut().clear();
        //     println!("testing split at: {split_at}, '{}'+'{}'", &json[0..split_at], &json[split_at..]);
        //     let mut parser = JsonStreamParser::new(|event| {
        //         events.borrow_mut().push(event.into())
        //     });
        //
        //     assert!(parser.parse(&bytes[0..split_at]).is_ok());
        //     assert!(parser.parse(&bytes[split_at..]).is_ok());
        //     *index.borrow_mut() = 0;
        //     test(&events.borrow(), get_next_idx);
        // }

        events.borrow_mut().clear();

        let mut parser = JsonStreamParser::new(|event| {
            events.borrow_mut().push(event.into())
        });
        assert!(parser.parse(&bytes).is_ok());

        *index.borrow_mut() = 0;
        test(&events.borrow(), get_next_idx);
    }

    // TODO add test for array of objects
}

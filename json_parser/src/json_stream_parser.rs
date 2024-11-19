#[derive(Debug)]
pub enum JsonEvent<'a> {
    Null,
    #[allow(dead_code)]
    Bool(bool),
    #[allow(dead_code)]
    Number(f64),
    #[allow(dead_code)]
    String(&'a str),
    #[allow(dead_code)]
    StartObject,
    #[allow(dead_code)]
    EndObject,
    #[allow(dead_code)]
    StartArray,
    #[allow(dead_code)]
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
    state: ParserState,
}

// TODO:
// 1. measure performance
// 2. create producer/consumer
// 3. Generate test data

#[derive(Debug)]
pub enum JsonStreamParseError {
    UnexpectedEndOfInput,
    #[allow(dead_code)]
    UnexpectedEndOfString,
    #[allow(dead_code)]
    UnexpectedChar(char),
    #[allow(dead_code)]
    InvalidArray(String),
    #[allow(dead_code)]
    InvalidObject(String),
    #[allow(dead_code)]
    InvalidLiteral(String),
    InvalidBoolean,
    #[allow(dead_code)]
    InvalidNumber,
}

// keeps position in the buffer
enum ParserState {
    Initial,
    #[allow(dead_code)]
    ParsingKey,
    ParsingString,
    #[allow(dead_code)]
    ParsingNum,
    ParsingTrue,
    ParsingFalse,
    ParsingNull,
}

impl<'a, F> JsonStreamParser<F>
where
    F: FnMut(JsonEvent),
{
    pub fn new(callback: F) -> Self {
        // TODO: play with initial size
        let buffer = Vec::<u8>::with_capacity(1024); // 1k
        let start_pos = 0;
        let offset = 0;
        let state = ParserState::Initial;
        Self {
            callback,
            buffer,
            start_pos,
            offset,
            state,
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

    pub fn parse(&mut self, chunk: &[u8]) -> Result<(), JsonStreamParseError> {
        self.append_buffer(chunk);
        self.skip_whitespace();
        self.parse_value()?;
        Ok(())
    }

    fn parse_value(&mut self) -> Result<bool, JsonStreamParseError> {
        match self.state {
            ParserState::Initial => {
                match self.peek_char() {
                    Some('n') => {
                        self.state = ParserState::ParsingNull;
                        self.parse_null()
                    }
                    Some('t') => {
                        self.state = ParserState::ParsingTrue;
                        self.parse_true()
                    }
                    Some('f') => {
                        self.state = ParserState::ParsingFalse;
                        self.parse_false()
                    }
                    Some('"') => {
                        self.state = ParserState::ParsingString;
                        self.consume_char('"')?;
                        self.start_pos += 1;
                        self.parse_string(false)
                    }
                    Some('[') => {
                        todo!() // self.parse_array()
                    }
                    Some('{') => {
                        todo!() // self.parse_object()
                    }
                    // Some(c) if c.is_digit(10) || c == '-' => self.parse_number(),
                    Some(c) => Err(JsonStreamParseError::UnexpectedChar(c)),
                    None => Err(JsonStreamParseError::UnexpectedEndOfInput),
                }
            }
            ParserState::ParsingKey => {
                todo!()
            }
            ParserState::ParsingString => {
                self.parse_string(false)
            }
            ParserState::ParsingNum => {
                todo!()
            }
            ParserState::ParsingTrue => {
                self.parse_true()
            }
            ParserState::ParsingFalse => {
                self.parse_false()
            }
            ParserState::ParsingNull => {
                self.parse_null()
            }
        }
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

    #[allow(dead_code)]
    fn parse_false(&mut self) -> Result<bool, JsonStreamParseError> {
        let complete = self.expect_literal("false").map_err(|_| JsonStreamParseError::InvalidBoolean)?;
        if complete {
            (self.callback)(JsonEvent::Bool(true));
            self.start_pos = self.offset;
        }
        Ok(complete)
    }

    // TODO review
    // fn parse_number(&mut self) -> Result<(), JsonStreamParseError> {
    //     let mut num = String::new();
    //
    //     while let Some(c) = self.peek_char() {
    //         if c.is_digit(10) || c == '.' || c == '-' || c == 'e' || c == 'E' {
    //             num.push(self.next_char().unwrap());
    //         } else {
    //             break;
    //         }
    //     }
    //
    //     match num.parse::<f64>() {
    //         Ok(n) => {
    //             (self.callback)(JsonEvent::Number(n));
    //             Ok(())
    //         }
    //         Err(_) => Err(JsonStreamParseError::InvalidNumber),
    //     }
    // }

    // TODO handle escaped characters
    fn parse_string(&mut self, _is_key: bool) -> Result<bool, JsonStreamParseError> {
        while let Some(c) = self.next_char() {
            if c == '"' {
    // TODO
    //             if is_key {
    //                 (self.callback)(JsonEvent::Key(text));
    //             }
                let bytes = &self.buffer[self.start_pos..(self.offset - 1)];
                let slice = std::str::from_utf8(bytes).expect(""); // TODO avoid expects
                (self.callback)(JsonEvent::String(slice));
                self.start_pos = self.offset;

                return Ok(true);
            }
        }

        Ok(false)
    }

    // TODO review
    // fn parse_array(&mut self) -> Result<(), JsonStreamParseError> {
    //     self.consume_char('[')?;
    //     (self.callback)(JsonEvent::StartArray);
    //
    //     loop {
    //         self.skip_whitespace();
    //         if self.peek_char() == Some(']') {
    //             self.consume_char(']')?;
    //             (self.callback)(JsonEvent::EndArray);
    //             break;
    //         }
    //
    //         self.parse_value()?;
    //         self.skip_whitespace();
    //
    //         if self.peek_char() == Some(',') {
    //             self.consume_char(',')?;
    //         } else if self.peek_char() != Some(']') {
    //             return Err(JsonStreamParseError::InvalidArray("Expected ',' or ']'".into()));
    //         }
    //     }
    //
    //     Ok(())
    // }

    // TODO review
    // fn parse_object(&mut self) -> Result<(), JsonStreamParseError> {
    //     self.consume_char('{')?;
    //     (self.callback)(JsonEvent::StartObject);
    //
    //     loop {
    //         self.skip_whitespace();
    //         if self.peek_char() == Some('}') {
    //             self.consume_char('}')?;
    //             (self.callback)(JsonEvent::EndObject);
    //             break;
    //         }
    //
    //         self.parse_string(true)?; // Обрабатываем ключ
    //         self.skip_whitespace();
    //         self.consume_char(':')?;
    //         self.skip_whitespace();
    //
    //         self.parse_value()?; // Обрабатываем значение
    //         self.skip_whitespace();
    //
    //         if self.peek_char() == Some(',') {
    //             self.consume_char(',')?;
    //         } else if self.peek_char() != Some('}') {
    //             return Err(JsonStreamParseError::InvalidObject("Expected ',' or '}'".into()));
    //         }
    //     }
    //
    //     Ok(())
    // }

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

    fn consume_char(&mut self, expected: char) -> Result<(), JsonStreamParseError> {
         if self.next_char() == Some(expected) {
             Ok(())
         } else {
             return Err(JsonStreamParseError::InvalidLiteral(format!("Expected '{}'", expected)));
         }
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

    fn test_json_parser_element(element: &str, expected: OwningJsonEvent) {
        let json = element;

        let events: RefCell<Vec<OwningJsonEvent>> = RefCell::new(vec![]);

        let mut parser = JsonStreamParser::new(|event| {
            events.borrow_mut().push(event.into())
        });

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

        // for split_at in 1..json.len() {
        //     events.borrow_mut().clear();
        //     assert!(parser.parse(&bytes[0..split_at]).is_ok());
        //     assert!(parser.parse(&bytes[split_at..]).is_ok());
        //     *index.borrow_mut() = 0;
        //     test(&events.borrow(), expected.clone(), get_next_idx);
        // }

        events.borrow_mut().clear();

        assert!(parser.parse(&bytes).is_ok());

        *index.borrow_mut() = 0;
        test(&events.borrow(), expected.clone(), get_next_idx);
    }

    #[test]
    fn test_json_parser_null() {
        let literal = "null";
        let expected = OwningJsonEvent::Null;
        test_json_parser_element(literal, expected);
    }

    #[test]
    fn test_json_parser_true() {
        let literal = "true";
        let expected = OwningJsonEvent::Bool(true);
        test_json_parser_element(literal, expected);
    }

    #[test]
    fn test_json_parser_false() {
        let literal = "false";
        let expected = OwningJsonEvent::Bool(true);
        test_json_parser_element(literal, expected);
    }

    #[test]
    fn test_json_parser_string() {
        let literal = "\"test string\"";
        let expected = OwningJsonEvent::String("test string".into());
        test_json_parser_element(literal, expected);
    }

    // TODO add test for null
    #[test]
    fn test_json_parser_callbacks() {
        //     let json = r#"
        //     {
        //         "name": "Alice",
        //         "age": 30,
        //         "is_active": true,
        //         "married": false,
        //         "skills": ["Rust", "C++"]
        //     }
        // "#;
        //
        //     let mut events: Vec<OwningJsonEvent> = vec![];
        //
        //     let mut parser = JsonStreamParser::new(json, |event| {
        //         events.push(event.into())
        //     });
        //     assert!(parser.parse().is_ok());
        //
        //     let mut index = 0;
        //     let mut get_next_idx = || {
        //         let result = index;
        //         index += 1;
        //         result
        //     };
        //
        //     assert_eq!(&events[get_next_idx()], &OwningJsonEvent::StartObject);
        //
        //     assert_eq!(&events[get_next_idx()], &OwningJsonEvent::Key("name".to_string()));
        //     assert_eq!(&events[get_next_idx()], &OwningJsonEvent::String("Alice".to_string()));
        //
        //     assert_eq!(&events[get_next_idx()], &OwningJsonEvent::Key("age".to_string()));
        //     assert_eq!(&events[get_next_idx()], &OwningJsonEvent::Number(30.0));
        //
        //     assert_eq!(&events[get_next_idx()], &OwningJsonEvent::Key("is_active".to_string()));
        //     assert_eq!(&events[get_next_idx()], &OwningJsonEvent::Bool(true));
        //
        //     assert_eq!(&events[get_next_idx()], &OwningJsonEvent::Key("married".to_string()));
        //     assert_eq!(&events[get_next_idx()], &OwningJsonEvent::Bool(false));
        //
        //     assert_eq!(&events[get_next_idx()], &OwningJsonEvent::Key("skills".to_string()));
        //
        //     assert_eq!(&events[get_next_idx()], &OwningJsonEvent::StartArray);
        //     {
        //         assert_eq!(&events[get_next_idx()], &OwningJsonEvent::String("Rust".to_string()));
        //         assert_eq!(&events[get_next_idx()], &OwningJsonEvent::String("C++".to_string()));
        //     }
        //     assert_eq!(&events[get_next_idx()], &OwningJsonEvent::EndArray);
        //
        //     assert_eq!(&events[get_next_idx()], &OwningJsonEvent::EndObject);
    }
}

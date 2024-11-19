#[derive(Debug)]
pub enum JsonEvent<'a> {
    Null,
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
    // input: Chars<'a>,
    callback: F,
    buffer: Vec<u8>,
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
    InvalidNumber,
}

// keeps position in the buffer
struct ParserState {
    start_pos: usize, // TODO extract to stream parser
    curr_element: ParserStateElement,
}


enum ParserStateElement {
    Initial,
    ParsingKey,
    ParsingString,
    ParsingNum,
    ParsingBool,
    ParsingNull,
}

impl<'a, F> JsonStreamParser<F>
where
    F: FnMut(JsonEvent),
{
    pub fn new(callback: F) -> Self {
        let mut buffer = Vec::<u8>::with_capacity(1024); // 1k
        let offset = 0;
        let state = ParserState {
            start_pos: 0,
            curr_element: ParserStateElement::Initial,
        };
        Self {
            callback,
            buffer,
            offset,
            state,
        }
    }

    fn set_start_pos(&mut self, pos: usize) {
        self.state.start_pos = pos
    }

    fn append_buffer(&mut self, chunk: &[u8]) {
        let start_pos = self.state.start_pos;
        self.offset = self.buffer.len() - start_pos;
        self.buffer.drain(..start_pos);

        // TODO: use custom cache friendly reallocation algorithm?
        self.buffer.extend_from_slice(chunk);
        self.set_start_pos(0);
    }

    pub fn parse(&mut self, chunk: &[u8]) -> Result<(), JsonStreamParseError> {
        self.append_buffer(chunk);
        self.skip_whitespace();
        self.parse_value()?;
        Ok(())
    }

    fn parse_value(&mut self) -> Result<(), JsonStreamParseError> {
        match self.state.curr_element {
            ParserStateElement::Initial => {
                match self.peek_char() {
                    Some('n') => {
                        self.state.curr_element = ParserStateElement::ParsingNull;
                        self.parse_null()
                    }
                    Some('t') => {
                        self.parse_true()
                    }
                    Some('f') => {
                        todo!() // self.parse_false()
                    }
                    Some('"') => {
                        todo!() // self.parse_string(false)
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
            ParserStateElement::ParsingKey => {
                todo!()
            }
            ParserStateElement::ParsingString => {
                todo!()
            }
            ParserStateElement::ParsingNum => {
                todo!()
            }
            ParserStateElement::ParsingBool => {
                todo!()
            }
            ParserStateElement::ParsingNull => {
                self.parse_null()
            }
        }
    }

    fn parse_null(&mut self) -> Result<(), JsonStreamParseError> {
        let complete = self.expect_literal("null")?;
        if complete {
            (self.callback)(JsonEvent::Null);
            self.set_start_pos(self.offset);
        }
        Ok(())
    }

    fn parse_true(&mut self) -> Result<(), JsonStreamParseError> {
        let complete = self.expect_literal("true").map_err(|_| JsonStreamParseError::InvalidBoolean)?;
        if complete {
            (self.callback)(JsonEvent::Bool(true));
            self.set_start_pos(self.offset);
        }
        Ok(())
    }

    fn parse_false(&mut self) -> Result<(), JsonStreamParseError> {
        let complete = self.expect_literal("false").map_err(|_| JsonStreamParseError::InvalidBoolean)?;
        if complete {
            (self.callback)(JsonEvent::Bool(true));
            self.set_start_pos(self.offset);
        }
        Ok(())
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

    // TODO review
    // fn parse_string(&mut self, is_key: bool) -> Result<(), JsonStreamParseError> {
    //     self.consume_char('"')?;
    //     let start = self.input.as_str();
    //
    //     while let Some(c) = self.next_char() {
    //         if c == '"' {
    //             let end = self.input.as_str();
    //             let len = start.len() - end.len() - 1;
    //             let text = &start[..len];
    //
    //             if is_key {
    //                 (self.callback)(JsonEvent::Key(text));
    //             } else {
    //                 (self.callback)(JsonEvent::String(text));
    //             }
    //             return Ok(());
    //         }
    //
    //         if c == '\\' {
    //             self.next_char(); // Пропускаем экранированный символ
    //         }
    //     }
    //
    //     Err(JsonStreamParseError::UnexpectedEndOfString)
    // }

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

    fn increment_start_pos(&mut self) {
        self.set_start_pos(self.state.start_pos + 1);
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.next_char();
                self.increment_start_pos();
            } else {
                break;
            }
        }
    }

    // TODO review
    fn expect_literal(&mut self, literal: &str) -> Result<bool, JsonStreamParseError> {
        let start_pos = self.offset - self.state.start_pos;
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

    // // TODO review
    // fn consume_char(&mut self, expected: char) -> Result<(), JsonStreamParseError> {
    //     if self.next_char() == Some(expected) {
    //         Ok(())
    //     } else {
    //         return Err(JsonStreamParseError::InvalidLiteral(format!("Expected '{}'", expected)));
    //     }
    // }

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
    use super::*;

    #[derive(Debug, PartialEq)]
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

    #[test]
    fn test_json_parser_null() {
        let json = "null";

        let mut events: Vec<OwningJsonEvent> = vec![];

        let mut parser = JsonStreamParser::new(|event| {
            events.push(event.into())
        });

        let bytes = json.as_bytes();

        for split_at in 1..json.len() {
            assert!(parser.parse(&bytes[0..split_at]).is_ok());
            assert!(parser.parse(&bytes[split_at..]).is_ok());
        }

        assert!(parser.parse(bytes).is_ok());

        let mut index = 0;
        let mut get_next_idx = || {
            let result = index;
            index += 1;
            result
        };

        assert_eq!(&events[get_next_idx()], &OwningJsonEvent::Null);
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

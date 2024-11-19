

use std::str::Chars;

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

pub struct JsonStreamParser<'a, F>
where
    F: FnMut(JsonEvent<'a>),
{
    input: Chars<'a>,
    buffer: &'a str, // Ссылка на исходную строку
    callback: F,
}

impl<'a, F> JsonStreamParser<'a, F>
where
    F: FnMut(JsonEvent<'a>),
{
    pub fn new(input: &'a str, callback: F) -> Self {
        Self {
            input: input.chars(),
            buffer: input,
            callback,
        }
    }

    pub fn parse(&mut self) -> Result<(), String> {
        self.skip_whitespace();
        self.parse_value()?;
        Ok(())
    }

    fn parse_value(&mut self) -> Result<(), String> {
        match self.peek_char() {
            Some('n') => self.parse_null(),
            Some('t') | Some('f') => self.parse_bool(),
            Some('"') => self.parse_string(false),
            Some('[') => self.parse_array(),
            Some('{') => self.parse_object(),
            Some(c) if c.is_digit(10) || c == '-' => self.parse_number(),
            Some(c) => Err(format!("Unexpected character: {}", c)),
            None => Err("Unexpected end of input".into()),
        }
    }

    fn parse_null(&mut self) -> Result<(), String> {
        self.expect_literal("null")?;
        (self.callback)(JsonEvent::Null);
        Ok(())
    }

    fn parse_bool(&mut self) -> Result<(), String> {
        if self.expect_literal("true").is_ok() {
            (self.callback)(JsonEvent::Bool(true));
            Ok(())
        } else if self.expect_literal("false").is_ok() {
            (self.callback)(JsonEvent::Bool(false));
            Ok(())
        } else {
            Err("Invalid boolean".into())
        }
    }

    fn parse_number(&mut self) -> Result<(), String> {
        let start = self.input.as_str();
        let mut num = String::new();

        while let Some(c) = self.peek_char() {
            if c.is_digit(10) || c == '.' || c == '-' || c == 'e' || c == 'E' {
                num.push(self.next_char().unwrap());
            } else {
                break;
            }
        }

        match num.parse::<f64>() {
            Ok(n) => {
                (self.callback)(JsonEvent::Number(n));
                Ok(())
            }
            Err(_) => Err("Invalid number".into()),
        }
    }

    fn parse_string(&mut self, is_key: bool) -> Result<(), String> {
        self.consume_char('"')?;
        let start = self.input.as_str();

        while let Some(c) = self.next_char() {
            if c == '"' {
                let end = self.input.as_str();
                let len = start.len() - end.len() - 1;
                let text = &start[..len];

                if is_key {
                    (self.callback)(JsonEvent::Key(text));
                } else {
                    (self.callback)(JsonEvent::String(text));
                }
                return Ok(());
            }

            if c == '\\' {
                self.next_char(); // Пропускаем экранированный символ
            }
        }

        Err("Unexpected end of string".into())
    }

    fn parse_array(&mut self) -> Result<(), String> {
        self.consume_char('[')?;
        (self.callback)(JsonEvent::StartArray);

        loop {
            self.skip_whitespace();
            if self.peek_char() == Some(']') {
                self.consume_char(']')?;
                (self.callback)(JsonEvent::EndArray);
                break;
            }

            self.parse_value()?;
            self.skip_whitespace();

            if self.peek_char() == Some(',') {
                self.consume_char(',')?;
            } else if self.peek_char() != Some(']') {
                return Err("Expected ',' or ']'".into());
            }
        }

        Ok(())
    }

    fn parse_object(&mut self) -> Result<(), String> {
        self.consume_char('{')?;
        (self.callback)(JsonEvent::StartObject);

        loop {
            self.skip_whitespace();
            if self.peek_char() == Some('}') {
                self.consume_char('}')?;
                (self.callback)(JsonEvent::EndObject);
                break;
            }

            self.parse_string(true)?; // Обрабатываем ключ
            self.skip_whitespace();
            self.consume_char(':')?;
            self.skip_whitespace();

            self.parse_value()?; // Обрабатываем значение
            self.skip_whitespace();

            if self.peek_char() == Some(',') {
                self.consume_char(',')?;
            } else if self.peek_char() != Some('}') {
                return Err("Expected ',' or '}'".into());
            }
        }

        Ok(())
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c.is_whitespace() {
                self.next_char();
            } else {
                break;
            }
        }
    }

    fn expect_literal(&mut self, literal: &str) -> Result<(), String> {
        for expected in literal.chars() {
            if self.next_char() != Some(expected) {
                return Err(format!("Expected literal: {}", literal));
            }
        }
        Ok(())
    }

    fn consume_char(&mut self, expected: char) -> Result<(), String> {
        if self.next_char() == Some(expected) {
            Ok(())
        } else {
            Err(format!("Expected '{}'", expected))
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input.clone().next()
    }

    fn next_char(&mut self) -> Option<char> {
        self.input.next()
    }
}

fn main() {
    println!("Hello world");
}

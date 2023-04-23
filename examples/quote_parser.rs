use finite_state_machine::state_machine;
use std::fmt;

#[derive(Default)]
pub struct Config {
    quotes: Vec<char>,
}

pub struct Data<'a> {
    index: usize,
    found: Vec<&'a str>,
    text: Option<&'a str>,
    quote: Option<char>,
}

impl<'a> Data<'a> {
    fn include_char(&mut self) -> Result<(), &'static str> {
        self.index += 1;
        Ok(())
    }
    fn reset_index(&mut self) -> Result<(), &'static str> {
        self.text = match self.text {
            Some(text) => Some(&text[self.index..]),
            None => Err("text is empty")?,
        };
        self.index = 0;
        Ok(())
    }
    fn skip_char(&mut self) -> Result<(), &'static str> {
        if self.index != 0 {
            Err("index is not null, illegal")?
        }
        self.text = match self.text {
            Some(text) => Some(&text[1..]),
            None => Err("text is empty")?,
        };
        Ok(())
    }
    fn set_quote(&mut self) -> Result<(), &'static str> {
        if self.index != 0 {
            Err("index is not null, illegal")?
        }
        let quote = match self.text {
            Some(text) => text.as_bytes()[self.index],
            None => Err("text is empty, illegal")?,
        };
        self.quote = Some(quote as char);
        self.skip_char()?;
        Ok(())
    }
    fn store_included_chars(&mut self) -> Result<(), &'static str> {
        let quoted = match self.text {
            Some(text) => &text[..self.index],
            None => Err("text is empty")?,
        };
        self.found.push(quoted);
        self.reset_index()?;
        Ok(())
    }
}

impl<'a> fmt::Debug for Data<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let buffer = match self.text {
            Some(text) => text.get(..self.index),
            None => None,
        };
        f.debug_struct("raw self")
            .field("index", &self.index)
            .field("found", &self.found)
            .field("buffer", &buffer)
            .field("remaining text", &self.text)
            .finish()
    }
}

state_machine!(
    QuoteParser(Config, Data);
    Start {
        Begin => LeftQuote
    },
    LeftQuote {
        FoundQuote => RightQuote,
        NoQuote => LeftQuote,
        EndOfText => End
    },
    RightQuote {
        FoundBackslash => EscapeChar,
        FoundQuote => LeftQuote,
        NoQuote => RightQuote,
        EndOfText => End
    },
    EscapeChar {
        FoundElse => RightQuote
    }
);

use quote_parser::*;

impl QuoteParser {
    fn new(quotes: Vec<char>) -> QuoteParser {
        let machine = QuoteParser {
            config: Config { quotes },
        };
        machine
    }
    fn parse<'a>(&mut self, text: &'a str) -> Result<Vec<&'a str>, &'static str> {
        let state_data = Data {
            index: 0,
            found: vec![],
            text: Some(text),
            quote: None,
        };

        let result = self.run_to_end(state_data)?;
        Ok(result.found)
    }
}

impl<'a> StartTransitions<Data<'a>> for QuoteParser {
    fn illegal(&mut self) {}
    fn begin(&mut self, data: Data<'a>) -> Result<Data<'a>, &'static str> {
        Ok(data)
    }
}

impl<'a> LeftQuoteTransitions<Data<'a>> for QuoteParser {
    fn illegal(&mut self) {}
    fn end_of_text(&mut self, mut data: Data<'a>) -> Result<Data<'a>, &'static str> {
        Ok(data)
    }
    fn found_quote(&mut self, mut data: Data<'a>) -> Result<Data<'a>, &'static str> {
        data.set_quote()?;
        Ok(data)
    }
    fn no_quote(&mut self, mut data: Data<'a>) -> Result<Data<'a>, &'static str> {
        data.skip_char()?;
        Ok(data)
    }
}

impl<'a> RightQuoteTransitions<Data<'a>> for QuoteParser {
    fn illegal(&mut self) {}
    fn end_of_text(&mut self, mut data: Data<'a>) -> Result<Data<'a>, &'static str> {
        Err("unmatched quote")?
    }
    fn found_quote(&mut self, mut data: Data<'a>) -> Result<Data<'a>, &'static str> {
        data.store_included_chars()?;
        data.skip_char();
        Ok(data)
    }
    fn no_quote(&mut self, mut data: Data<'a>) -> Result<Data<'a>, &'static str> {
        data.include_char();
        Ok(data)
    }
    fn found_backslash(&mut self, mut data: Data<'a>) -> Result<Data<'a>, &'static str> {
        data.include_char();
        Ok(data)
    }
}

impl<'a> EscapeCharTransitions<Data<'a>> for QuoteParser {
    fn illegal(&mut self) {}
    fn found_else(&mut self, mut data: Data<'a>) -> Result<Data<'a>, &'static str> {
        data.include_char()?;
        Ok(data)
    }
}

impl<'a> Deciders<Data<'a>> for QuoteParser {
    fn start(&self, data: &Data) -> StartEvents {
        StartEvents::Begin
    }
    fn left_quote(&self, data: &Data) -> LeftQuoteEvents {
        let char = match data.text {
            Some(text) => text.as_bytes().get(data.index),
            None => return LeftQuoteEvents::Illegal("text is empty"),
        };
        match char {
            Some(c) if self.config.quotes.contains(&(*c as char)) => LeftQuoteEvents::FoundQuote,
            Some(_) => LeftQuoteEvents::NoQuote,
            None => LeftQuoteEvents::EndOfText,
        }
    }
    fn right_quote(&self, data: &Data) -> RightQuoteEvents {
        let quote = match data.quote {
            Some(q) => q,
            None => return RightQuoteEvents::Illegal("quote is empty"),
        };
        let char = match data.text {
            Some(text) => text.as_bytes().get(data.index),
            None => return RightQuoteEvents::Illegal("text is empty"),
        };
        match char {
            Some(c) if &(quote as u8) == c => RightQuoteEvents::FoundQuote,
            Some(c) if b'\\' == *c => RightQuoteEvents::FoundBackslash,
            Some(_) => RightQuoteEvents::NoQuote,
            None => RightQuoteEvents::EndOfText,
        }
    }
    fn escape_char(&self, data: &Data) -> EscapeCharEvents {
        let char = match data.text {
            Some(text) => text.as_bytes().get(data.index),
            None => return EscapeCharEvents::Illegal("text is empty"),
        };
        match char {
            Some(_) => EscapeCharEvents::FoundElse,
            None => EscapeCharEvents::Illegal("char is empty after escape character"),
        }
    }
}

fn main() {
    let mut parser = QuoteParser::new(vec!['\'', '"']);
    let input = "Hello 'World' from \"macro_rules!\". I can even do \"escaped quotes ->\\\"<-\"";
    println!("Finding quoted chars in: {}", input);
    let result = parser.parse(input);
    match result {
        Ok(data) => {
            for entry in data {
                println!("{}", entry);
            }
        }
        Err(message) => println!("Error: {:?}", message),
    };
}

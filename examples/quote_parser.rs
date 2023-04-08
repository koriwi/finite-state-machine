use std::fmt;

use finite_state_machine::state_machine;

#[derive(Default)]
pub struct Data<'a> {
    index: usize,
    found: Option<Vec<&'a str>>,
    quotes: Vec<char>,
    text: Option<&'a str>,
    quote: Option<char>,
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
    QuoteParser<'a>(Data<'a>);
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

impl<'a> QuoteParser<'a> {
    fn include_char(&mut self) -> Result<(), String> {
        self.data.index += 1;
        Ok(())
    }
    fn reset_index(&mut self) -> Result<(), String> {
        self.data.text = match self.data.text {
            Some(text) => Some(&text[self.data.index..]),
            None => Err("text is empty")?,
        };
        self.data.index = 0;
        Ok(())
    }
    fn skip_char(&mut self) -> Result<(), String> {
        if self.data.index != 0 {
            Err("index is not null, illegal")?
        }
        self.data.text = match self.data.text {
            Some(text) => Some(&text[1..]),
            None => Err("text is empty")?,
        };
        Ok(())
    }
    fn set_quote(&mut self) -> Result<(), String> {
        if self.data.index != 0 {
            Err("index is not null, illegal")?
        }
        let quote = match self.data.text {
            Some(text) => text.as_bytes()[self.data.index],
            None => Err("text is empty, illegal")?,
        };
        self.data.quote = Some(quote as char);
        self.skip_char()?;
        Ok(())
    }
    fn store_included_chars(&mut self) -> Result<(), String> {
        let quoted = match self.data.text {
            Some(text) => &text[..self.data.index],
            None => Err("text is empty")?,
        };
        match self.data.found.as_mut() {
            Some(found) => found.push(quoted),
            None => {
                self.data.found = Some(vec![quoted]);
            }
        };
        self.reset_index()?;
        Ok(())
    }
    fn new(quotes: Vec<char>) -> QuoteParser<'a> {
        let mut machine = QuoteParser::default();
        machine.data.quotes = quotes;
        machine
    }
    fn parse(&mut self, text: &'a String) -> Result<Option<Vec<&str>>, String> {
        self.data.text = Some(text);
        self.run()?;
        Ok(self.data.found.take())
    }
}

impl<'a> StartTransitions for QuoteParser<'a> {
    fn illegal(&mut self) {}
    fn begin(&mut self) -> Result<(), String> {
        Ok(())
    }
}

impl<'a> LeftQuoteTransitions for QuoteParser<'a> {
    fn illegal(&mut self) {}
    fn end_of_text(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn found_quote(&mut self) -> Result<(), String> {
        self.set_quote()?;
        Ok(())
    }
    fn no_quote(&mut self) -> Result<(), String> {
        self.skip_char()?;
        Ok(())
    }
}

impl<'a> RightQuoteTransitions for QuoteParser<'a> {
    fn illegal(&mut self) {}
    fn end_of_text(&mut self) -> Result<(), String> {
        Err("unmatched quote")?
    }
    fn found_quote(&mut self) -> Result<(), String> {
        self.store_included_chars()?;
        self.skip_char()
    }
    fn no_quote(&mut self) -> Result<(), String> {
        self.include_char()
    }
    fn found_backslash(&mut self) -> Result<(), String> {
        self.include_char()
    }
}

impl<'a> EscapeCharTransitions for QuoteParser<'a> {
    fn illegal(&mut self) {}
    fn found_else(&mut self) -> Result<(), String> {
        self.include_char()
    }
}

impl<'a> Deciders for QuoteParser<'a> {
    fn start(&self) -> StartEvents {
        StartEvents::Begin
    }
    fn left_quote(&self) -> LeftQuoteEvents {
        let char = match self.data.text {
            Some(text) => text.as_bytes().get(self.data.index),
            None => return LeftQuoteEvents::Illegal,
        };
        match char {
            Some(c) if self.data.quotes.contains(&(*c as char)) => LeftQuoteEvents::FoundQuote,
            Some(_) => LeftQuoteEvents::NoQuote,
            None => LeftQuoteEvents::EndOfText,
        }
    }
    fn right_quote(&self) -> RightQuoteEvents {
        let quote = match self.data.quote {
            Some(q) => q,
            None => return RightQuoteEvents::Illegal,
        };
        let char = match self.data.text {
            Some(text) => text.as_bytes().get(self.data.index),
            None => return RightQuoteEvents::Illegal,
        };
        match char {
            Some(c) if &(quote as u8) == c => RightQuoteEvents::FoundQuote,
            Some(c) if b'\\' == *c => RightQuoteEvents::FoundBackslash,
            Some(_) => RightQuoteEvents::NoQuote,
            None => RightQuoteEvents::EndOfText,
        }
    }
    fn escape_char(&self) -> EscapeCharEvents {
        let char = match self.data.text {
            Some(text) => text.as_bytes().get(self.data.index),
            None => return EscapeCharEvents::Illegal,
        };
        match char {
            Some(_) => EscapeCharEvents::FoundElse,
            None => EscapeCharEvents::Illegal,
        }
    }
}

fn main() {
    let mut parser = QuoteParser::new(vec!['\'', '"']);
    let input = "Hello 'World' from \"macro_rules!\". I can even do \"escaped quotes ->\\\"<-\""
        .to_string();
    println!("Finding quoted chars in: {}", input);
    let result = parser.parse(&input);
    match result {
        Ok(Some(data)) => {
            for entry in data {
                println!("{}", entry);
            }
        }
        Ok(None) => println!("Success but found nothing"),
        Err(message) => println!("Error: {:?}", message),
    };
}

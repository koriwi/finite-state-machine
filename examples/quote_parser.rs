use finite_state_machine::state_machine;

#[derive(Debug, Clone, PartialEq)]
struct Data {
    buffer: String,
    quotes: Vec<char>,
    text: String,
    char: Option<char>,
    found: Vec<String>,
    quote: Option<char>,
    index: usize,
}

state_machine!(
    QuoteParser(Data);
    Start {
        Begin => LeftQuote
    },
    LeftQuote {
        FoundQuote => RightQuote,
        NoQuote => LeftQuote,
        EndOfText => End
    },
    RightQuote {
        FoundQuote => LeftQuote,
        NoQuote => RightQuote,
        EndOfText => End
    }
);

impl QuoteParser {
    fn reset(&mut self) {
        self.data.char = None;
        self.data.index = 0;
        self.data.buffer = String::new();
        self.data.quote = None;
        self.data.text = String::new();
    }
    fn next_char(&mut self) {
        self.data.index += 1;
        self.data.char = self.data.text.chars().nth(self.data.index);
    }
    fn new(quotes: Vec<char>) -> QuoteParser {
        QuoteParser {
            data: Data {
                buffer: String::new(),
                quotes,
                text: String::new(),
                char: None,
                found: Vec::new(),
                quote: None,
                index: 0,
            },
            state: State::Start,
        }
    }
    fn parse(&mut self, text: String) -> Result<Data, String> {
        self.data.text = text;
        self.run()
    }
}

impl Transitions for QuoteParser {
    fn left_quote_impossible(&mut self) {}
    fn right_quote_impossible(&mut self) {}
    fn start_impossible(&mut self) {}
    fn start_begin(&mut self) -> Result<(), String> {
        self.data.char = self.data.text.chars().nth(self.data.index);
        Ok(())
    }
    fn left_quote_end_of_text(&mut self) -> Result<(), String> {
        self.reset();
        Ok(())
    }
    fn right_quote_end_of_text(&mut self) -> Result<(), String> {
        self.reset();
        Ok(())
    }
    fn left_quote_found_quote(&mut self) -> Result<(), String> {
        self.data.quote = self.data.char;
        self.next_char();
        Ok(())
    }
    fn left_quote_no_quote(&mut self) -> Result<(), String> {
        self.next_char();
        Ok(())
    }
    fn right_quote_found_quote(&mut self) -> Result<(), String> {
        self.data.found.push(self.data.buffer.clone());
        self.data.buffer = String::new();
        self.data.quote = None;
        self.next_char();
        Ok(())
    }
    fn right_quote_no_quote(&mut self) -> Result<(), String> {
        let char = self.data.char.ok_or(String::from("char is gone"))?;
        self.data.buffer.push(char);
        self.next_char();
        Ok(())
    }
}

impl StateActions for QuoteParser {
    fn run_start(&self) -> StartEvents {
        StartEvents::Begin
    }
    fn run_left_quote(&self) -> LeftQuoteEvents {
        let char = match self.data.char {
            Some(c) => c,
            None => return LeftQuoteEvents::EndOfText,
        };
        match self.data.quotes.contains(&char) {
            true => LeftQuoteEvents::FoundQuote,
            false => LeftQuoteEvents::NoQuote,
        }
    }
    fn run_right_quote(&self) -> RightQuoteEvents {
        let quote = match self.data.quote {
            Some(q) => q,
            None => return RightQuoteEvents::Impossible,
        };
        let char = match self.data.char {
            Some(c) => c,
            None => return RightQuoteEvents::EndOfText,
        };
        match char == quote {
            true => RightQuoteEvents::FoundQuote,
            false => RightQuoteEvents::NoQuote,
        }
    }
}

fn main() {
    let mut parser = QuoteParser::new(vec!['\'', '"']);
    let input = "Hello 'World' from \"macro_rules!\"".to_string();
    println!("Finding quoted chars in: {}", input);
    let result = parser.parse(input);
    match result {
        Ok(data) => println!("Found {:?}", data.found),
        Err(message) => println!("Error, but found {:?}", message),
    };
}

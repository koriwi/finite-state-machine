use finite_state_machine::state_machine;
use std::{error::Error, str::Chars, time::SystemTime};

type CSVRow = Vec<Option<String>>;
#[derive(Debug, PartialEq)]
struct CSVData {
    column_names: Vec<String>,
    rows: Vec<CSVRow>,
}
impl CSVData {
    fn new(column_names: Vec<String>, rows: Vec<CSVRow>) -> Self {
        CSVData { column_names, rows }
    }
    fn push_column(&mut self, column_name: String) -> Result<(), String> {
        self.column_names.push(column_name);
        Ok(())
    }
    fn push_value(&mut self, value: Option<String>) -> Result<(), String> {
        self.rows
            .last_mut()
            .ok_or("rows cannot be empty, impossible")?
            .push(value);
        Ok(())
    }
    fn add_empty_row(&mut self) -> Result<(), String> {
        match self.rows.last() {
            Some(row) => {
                if row.len() != self.column_names.len() {
                    return Err(format!(
                        "row length {} does not match column length {}",
                        row.len(),
                        self.column_names.len()
                    ))?;
                }
            }
            None => {}
        }
        self.rows.push(vec![]);
        Ok(())
    }
}

#[derive(Debug)]
pub struct Data<'a> {
    char: Option<char>,
    quote: Option<char>,
    input: Chars<'a>,
    delimiter: char,
    field_buffer: Option<String>,
    parsed_csv: Option<CSVData>,
}

impl<'a> Default for Data<'a> {
    fn default() -> Self {
        Data {
            char: None,
            quote: None,
            input: "".chars(),
            delimiter: ',',
            field_buffer: None,
            parsed_csv: None,
        }
    }
}

state_machine!(
    CsvParser<'a>(Data<'a>);
    Start {
        Begin => FindHeaderDelimiter
    },
    FindHeaderDelimiter {
        FoundDelimiter => FindHeaderDelimiter,
        FoundLeftQuote => FindHeaderRightQuote,
        FoundNewLine => FindBodyDelimiter,
        FoundElse => FindHeaderDelimiter,
        Empty => End
    },
    FindHeaderRightQuote {
        FoundRightQuote => FindHeaderDelimiter,
        FoundElse => FindHeaderRightQuote
    },
    FindBodyDelimiter {
        FoundDelimiter => FindBodyDelimiter,
        FoundLeftQuote => FindBodyRightQuote,
        FoundNewLine => FindBodyDelimiter,
        FoundElse => FindBodyDelimiter,
        Empty => End
    },
    FindBodyRightQuote {
        FoundRightQuote => FindBodyDelimiter,
        FoundElse => FindBodyRightQuote
    }
);

use csv_parser::*;

impl<'a> CsvParser<'a> {
    fn new(delimiter: char) -> Self {
        let mut parser = CsvParser::default();
        parser.data.delimiter = delimiter;
        parser
    }
    fn push_field_as_value_to_row(&mut self) -> Result<(), String> {
        let field = self
            .data
            .field_buffer
            .take()
            .ok_or("field_buffer is None, impossible")?;
        let parsed_csv = self
            .data
            .parsed_csv
            .as_mut()
            .ok_or("parsed_csv is undefined, impossible")?;
        match field.len() {
            0 => parsed_csv.push_value(None),
            _ => parsed_csv.push_value(Some(field)),
        }
    }
    fn push_field_as_value_to_columns(&mut self) -> Result<(), String> {
        let column = self
            .data
            .field_buffer
            .take()
            .ok_or("field_buffer is None, impossible")?;
        if column.len() == 0 {
            return Err("column name cannot be empty")?;
        }
        self.data
            .parsed_csv
            .as_mut()
            .ok_or("parsed_csv is undefined, impossible")?
            .push_column(column)?;
        Ok(())
    }
    fn add_empty_row(&mut self) -> Result<(), String> {
        self.data
            .parsed_csv
            .as_mut()
            .ok_or("parsed_csv is undefined, impossible")?
            .add_empty_row()?;
        Ok(())
    }
    fn store_char_in_field_buffer(&mut self) -> Result<(), String> {
        let char = self.data.char.ok_or("char cannot disappear")?;
        match self.data.field_buffer {
            Some(ref mut field_buffer) => field_buffer.push(char),
            None => self.data.field_buffer = Some(char.to_string()),
        }
        Ok(())
    }
    fn set_next_char(&mut self) {
        self.data.char = self.data.input.next();
    }
    fn parse(&mut self, text: &'a String) -> Result<Option<CSVData>, Box<dyn Error>> {
        self.data.input = text.chars();
        self.run()?;
        Ok(self.data.parsed_csv.take())
    }
}

impl<'a> FindBodyDelimiterTransitions for CsvParser<'a> {
    fn impossible(&mut self) {}
    fn found_new_line(&mut self) -> Result<(), String> {
        self.push_field_as_value_to_row()?;
        self.add_empty_row()?;
        self.set_next_char();
        Ok(())
    }
    fn found_else(&mut self) -> Result<(), String> {
        self.store_char_in_field_buffer()?;
        self.set_next_char();
        Ok(())
    }
    fn found_delimiter(&mut self) -> Result<(), String> {
        self.push_field_as_value_to_row()?;
        self.set_next_char();
        Ok(())
    }
    fn empty(&mut self) -> Result<(), String> {
        match self.data.field_buffer {
            Some(ref field_buffer) => {
                if field_buffer.len() > 0 {
                    self.push_field_as_value_to_row()?;
                };
            }
            None => {}
        }
        Ok(())
    }
    fn found_left_quote(&mut self) -> Result<(), String> {
        self.data.quote = self.data.char;
        self.set_next_char();
        Ok(())
    }
}

impl<'a> StartTransitions for CsvParser<'a> {
    fn impossible(&mut self) {}
    fn begin(&mut self) -> Result<(), String> {
        self.data.char = self.data.input.next();
        self.data.parsed_csv = Some(CSVData::new(vec![], vec![]));
        Ok(())
    }
}

impl<'a> FindBodyRightQuoteTransitions for CsvParser<'a> {
    fn impossible(&mut self) {}
    fn found_right_quote(&mut self) -> Result<(), String> {
        self.data.quote = None;
        self.set_next_char();
        Ok(())
    }
    fn found_else(&mut self) -> Result<(), String> {
        self.store_char_in_field_buffer()?;
        self.set_next_char();
        Ok(())
    }
}

impl<'a> FindHeaderDelimiterTransitions for CsvParser<'a> {
    fn impossible(&mut self) {}
    fn found_else(&mut self) -> Result<(), String> {
        self.store_char_in_field_buffer()?;
        self.set_next_char();
        Ok(())
    }
    fn found_delimiter(&mut self) -> Result<(), String> {
        self.push_field_as_value_to_columns()?;
        self.set_next_char();
        Ok(())
    }
    fn empty(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn found_left_quote(&mut self) -> Result<(), String> {
        self.data.quote = self.data.char;
        self.set_next_char();
        Ok(())
    }
    fn found_new_line(&mut self) -> Result<(), String> {
        self.push_field_as_value_to_columns()?;
        self.add_empty_row()?;
        self.set_next_char();
        Ok(())
    }
}

impl<'a> FindHeaderRightQuoteTransitions for CsvParser<'a> {
    fn impossible(&mut self) {}

    fn found_else(&mut self) -> Result<(), String> {
        FindHeaderDelimiterTransitions::found_else(self)
    }
    fn found_right_quote(&mut self) -> Result<(), String> {
        self.data.quote = None;
        self.set_next_char();
        Ok(())
    }
}

impl<'a> Deciders for CsvParser<'a> {
    fn start(&self) -> StartEvents {
        StartEvents::Begin
    }
    fn find_header_delimiter(&self) -> FindHeaderDelimiterEvents {
        let char = match self.data.char {
            Some(c) => c,
            None => return FindHeaderDelimiterEvents::Empty,
        };
        match char {
            c if c == self.data.delimiter => FindHeaderDelimiterEvents::FoundDelimiter,
            '"' => FindHeaderDelimiterEvents::FoundLeftQuote,
            '\'' => FindHeaderDelimiterEvents::FoundLeftQuote,
            '\n' => FindHeaderDelimiterEvents::FoundNewLine,
            _ => FindHeaderDelimiterEvents::FoundElse,
        }
    }
    fn find_header_right_quote(&self) -> FindHeaderRightQuoteEvents {
        let char = match self.data.char {
            Some(c) => c,
            None => return FindHeaderRightQuoteEvents::Illegal,
        };
        let quote = match self.data.quote {
            Some(c) => c,
            None => return FindHeaderRightQuoteEvents::Illegal,
        };
        match char {
            c if c == quote => FindHeaderRightQuoteEvents::FoundRightQuote,
            _ => FindHeaderRightQuoteEvents::FoundElse,
        }
    }
    fn find_body_delimiter(&self) -> FindBodyDelimiterEvents {
        let char = match self.data.char {
            Some(c) => c,
            None => return FindBodyDelimiterEvents::Empty,
        };
        match char {
            c if c == self.data.delimiter => FindBodyDelimiterEvents::FoundDelimiter,
            '"' => FindBodyDelimiterEvents::FoundLeftQuote,
            '\'' => FindBodyDelimiterEvents::FoundLeftQuote,
            '\n' => FindBodyDelimiterEvents::FoundNewLine,
            _ => FindBodyDelimiterEvents::FoundElse,
        }
    }
    fn find_body_right_quote(&self) -> FindBodyRightQuoteEvents {
        let char = match self.data.char {
            Some(c) => c,
            None => return FindBodyRightQuoteEvents::Illegal,
        };
        let quote = match self.data.quote {
            Some(c) => c,
            None => return FindBodyRightQuoteEvents::Illegal,
        };
        match char {
            c if c == quote => FindBodyRightQuoteEvents::FoundRightQuote,
            '\n' => FindBodyRightQuoteEvents::Illegal,
            _ => FindBodyRightQuoteEvents::FoundElse,
        }
    }
}

fn main() {
    let mut csv_parser = CsvParser::new(',');
    let text = std::fs::read_to_string("./examples/small.csv").expect("no file");
    let now = SystemTime::now();
    let result = csv_parser.parse(&text);
    match result {
        Ok(Some(ref data)) => {
            println!(
                "finished {}mb in: {:.2}s",
                text.len() / 1024 / 1024,
                now.elapsed().expect("could not get time").as_secs_f32()
            );
            println!("columns: {:?}", data.column_names);
            println!("row 9999: {:?}", data.rows.get(9999).expect("no row 9999"));
        }
        _ => println!("Error"),
    }
}

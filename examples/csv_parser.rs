use finite_state_machine::state_machine;
use std::{error::Error, time::SystemTime};

type CSVRow<'a> = Vec<Option<&'a str>>;
#[derive(Debug, PartialEq)]
struct CSVData<'a> {
    column_names: Vec<&'a str>,
    rows: Vec<CSVRow<'a>>,
}
impl<'a> CSVData<'a> {
    fn new(column_names: Vec<&'a str>, rows: Vec<CSVRow<'a>>) -> Self {
        CSVData { column_names, rows }
    }
    fn push_column(&mut self, column_name: &'a str) -> Result<(), String> {
        self.column_names.push(column_name);
        Ok(())
    }
    fn push_value(&mut self, value: Option<&'a str>) -> Result<(), String> {
        self.rows
            .last_mut()
            .ok_or("rows cannot be empty, impossible")?
            .push(value);
        Ok(())
    }
    fn add_empty_row(&mut self) -> Result<(), String> {
        if let Some(row) = self.rows.last() {
            if row.len() != self.column_names.len() {
                return Err(format!(
                    "row length {} does not match column length {}",
                    row.len(),
                    self.column_names.len()
                ));
            }
        }
        self.rows.push(vec![]);
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct Data<'a> {
    trim_quotes: bool,
    quote: Option<&'a u8>,
    input: Option<&'a str>,
    index: usize,
    delimiter: u8,
    parsed_csv: Option<CSVData<'a>>,
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

impl<'a> StartTransitions for CsvParser<'a> {
    fn illegal(&mut self) {}
    fn begin(&mut self) -> Result<(), String> {
        self.data.parsed_csv = Some(CSVData::new(vec![], vec![]));
        Ok(())
    }
}

impl<'a> FindHeaderDelimiterTransitions for CsvParser<'a> {
    fn illegal(&mut self) {}
    fn found_else(&mut self) -> Result<(), String> {
        self.store_char()?;
        Ok(())
    }
    fn found_delimiter(&mut self) -> Result<(), String> {
        self.store_cs_value(true)?;
        Ok(())
    }
    fn empty(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn found_left_quote(&mut self) -> Result<(), String> {
        self.set_quote()?;
        self.store_char()?;
        Ok(())
    }
    fn found_new_line(&mut self) -> Result<(), String> {
        self.store_cs_value(true)?;
        self.add_empty_row()?;
        Ok(())
    }
}

impl<'a> FindHeaderRightQuoteTransitions for CsvParser<'a> {
    fn illegal(&mut self) {}

    fn found_else(&mut self) -> Result<(), String> {
        FindHeaderDelimiterTransitions::found_else(self)
    }
    fn found_right_quote(&mut self) -> Result<(), String> {
        self.store_char()?;
        Ok(())
    }
}

impl<'a> FindBodyDelimiterTransitions for CsvParser<'a> {
    fn illegal(&mut self) {}
    fn found_new_line(&mut self) -> Result<(), String> {
        self.store_cs_value(false)?;
        self.add_empty_row()?;
        Ok(())
    }
    fn found_else(&mut self) -> Result<(), String> {
        self.store_char()?;
        Ok(())
    }
    fn found_delimiter(&mut self) -> Result<(), String> {
        self.store_cs_value(false)?;
        Ok(())
    }
    fn empty(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn found_left_quote(&mut self) -> Result<(), String> {
        self.set_quote()?;
        self.store_char()?;
        Ok(())
    }
}

impl<'a> FindBodyRightQuoteTransitions for CsvParser<'a> {
    fn illegal(&mut self) {}
    fn found_right_quote(&mut self) -> Result<(), String> {
        self.data.quote = None;
        self.store_char()?;
        Ok(())
    }
    fn found_else(&mut self) -> Result<(), String> {
        self.store_char()?;
        Ok(())
    }
}

impl<'a> Deciders for CsvParser<'a> {
    fn start(&self) -> StartEvents {
        StartEvents::Begin
    }
    fn find_header_delimiter(&self) -> FindHeaderDelimiterEvents {
        let char = match self.data.input {
            Some(c) => c.as_bytes()[self.data.index],
            None => return FindHeaderDelimiterEvents::Empty,
        };
        match char {
            c if c == self.data.delimiter => FindHeaderDelimiterEvents::FoundDelimiter,
            b'"' => FindHeaderDelimiterEvents::FoundLeftQuote,
            b'\'' => FindHeaderDelimiterEvents::FoundLeftQuote,
            b'\n' => FindHeaderDelimiterEvents::FoundNewLine,
            _ => FindHeaderDelimiterEvents::FoundElse,
        }
    }
    fn find_header_right_quote(&self) -> FindHeaderRightQuoteEvents {
        let char = match self.data.input {
            Some(c) if !c.is_empty() => c.as_bytes()[self.data.index],
            _ => return FindHeaderRightQuoteEvents::Illegal,
        };
        let quote = match self.data.quote {
            Some(c) => c,
            None => return FindHeaderRightQuoteEvents::Illegal,
        };
        match char {
            c if &c == quote => FindHeaderRightQuoteEvents::FoundRightQuote,
            _ => FindHeaderRightQuoteEvents::FoundElse,
        }
    }
    fn find_body_delimiter(&self) -> FindBodyDelimiterEvents {
        let char = match self.data.input {
            Some(c) if !c.is_empty() => c.as_bytes()[self.data.index],
            _ => return FindBodyDelimiterEvents::Empty,
        };
        match char {
            c if c == self.data.delimiter => FindBodyDelimiterEvents::FoundDelimiter,
            b'"' => FindBodyDelimiterEvents::FoundLeftQuote,
            b'\'' => FindBodyDelimiterEvents::FoundLeftQuote,
            b'\n' => FindBodyDelimiterEvents::FoundNewLine,
            _ => FindBodyDelimiterEvents::FoundElse,
        }
    }
    fn find_body_right_quote(&self) -> FindBodyRightQuoteEvents {
        let char = match self.data.input {
            Some(c) if !c.is_empty() => c.as_bytes()[self.data.index],
            _ => return FindBodyRightQuoteEvents::Illegal,
        };
        let quote = match self.data.quote {
            Some(c) => c,
            None => return FindBodyRightQuoteEvents::Illegal,
        };
        match char {
            c if &c == quote => FindBodyRightQuoteEvents::FoundRightQuote,
            b'\n' => FindBodyRightQuoteEvents::Illegal,
            _ => FindBodyRightQuoteEvents::FoundElse,
        }
    }
}

impl<'a> CsvParser<'a> {
    fn new(delimiter: char, trim_quotes: bool) -> Self {
        let mut parser = CsvParser::default();
        parser.data.delimiter = delimiter as u8;
        parser.data.trim_quotes = trim_quotes;
        parser
    }
    fn store_cs_value(&mut self, is_header: bool) -> Result<(), String> {
        let value = &self.data.input.ok_or("input is empty")?[..self.data.index];
        let value = match self.data.trim_quotes {
            true => value.trim_matches('"').trim_matches('\''),
            false => value,
        };
        let parsed_csv = self
            .data
            .parsed_csv
            .as_mut()
            .ok_or("parsed_csv is undefined, impossible")?;
        if is_header {
            if value.is_empty() {
                return Err("value cannot be empty in header")?;
            }
            parsed_csv.push_column(value)?;
        } else {
            if value.is_empty() {
                parsed_csv.push_value(None)?;
            } else {
                parsed_csv.push_value(Some(value))?;
            }
        };
        self.skip_char_and_set_start()?;
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
    fn store_char(&mut self) -> Result<(), String> {
        self.data.index += 1;
        Ok(())
    }
    fn skip_char_and_set_start(&mut self) -> Result<(), String> {
        self.data.input = Some(&self.data.input.ok_or("input is empty")?[self.data.index + 1..]);
        self.data.index = 0;
        Ok(())
    }
    fn set_quote(&mut self) -> Result<(), String> {
        let input = self.data.input.ok_or("input is empty")?;
        self.data.quote = input.as_bytes().get(self.data.index);
        Ok(())
    }
    fn parse(&mut self, text: &'a String) -> Result<Option<CSVData>, Box<dyn Error>> {
        self.data.input = Some(text);
        self.run()?;
        Ok(self.data.parsed_csv.take())
    }
}

fn main() {
    let mut csv_parser = CsvParser::new(',', true);
    let text = std::fs::read_to_string("./examples/small.csv").expect("no file");
    let now = SystemTime::now();
    let result = csv_parser.parse(&text);
    match result {
        Ok(Some(ref data)) => {
            println!(
                "finished {:.2}mb in: {:?}",
                (text.len() as f32) / 1024f32 / 1024f32,
                now.elapsed().expect("could not get time")
            );
            println!("columns: {:?}", data.column_names);
            println!("row 9999: {:?}", data.rows.get(9999).expect("no row 9999"));
        }
        Err(e) => println!("Error {}", e),
        _ => println!("no data"),
    }
}

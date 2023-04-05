use std::str::Chars;

use finite_state_machine::state_machine;

#[derive(Debug, Clone, PartialEq)]
struct CSVRow {
    column_names: Vec<String>,
    columns: Vec<String>,
}
impl CSVRow {
    fn new(column_names: Vec<String>, columns: Vec<String>) -> Self {
        CSVRow {
            column_names,
            columns,
        }
    }
    fn len(&self) -> usize {
        self.columns.len()
    }
    fn get(&self, column_name: String) -> Option<&String> {
        let index = self.column_names.iter().position(|x| x == &column_name);
        match index {
            Some(index) => self.columns.get(index),
            None => None,
        }
    }
    fn push(&mut self, column: String) {
        self.columns.push(column);
    }
}

#[derive(Debug, Clone)]
pub struct Data<'a> {
    char: Option<char>,
    quote: Option<char>,
    input: Chars<'a>,
    delimiter: char,
    column_names: Vec<String>,
    field_buffer: String,
    rows: Vec<CSVRow>,
}

impl<'a> Default for Data<'a> {
    fn default() -> Self {
        Data {
            char: None,
            quote: None,
            input: "".chars(),
            delimiter: ',',
            column_names: Vec::new(),
            field_buffer: String::new(),
            rows: Vec::new(),
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
    fn push_field_to_row(&mut self) -> Result<(), String> {
        let field = self.data.field_buffer.clone();
        self.data.field_buffer = String::new();
        self.get_last_row()?.push(field);
        Ok(())
    }
    fn get_last_row(&mut self) -> Result<&mut CSVRow, String> {
        self.data
            .rows
            .last_mut()
            .ok_or("last_row is undefined, impossible".to_string())
    }
    fn get_last_column(&mut self) -> Result<&mut String, String> {
        self.data
            .column_names
            .last_mut()
            .ok_or("last_column is undefined, impossible".to_string())
    }
    fn store_char_in_field_buffer(&mut self) -> Result<(), String> {
        let char = self.data.char.ok_or("char cannot disappear".to_string())?;
        self.data.field_buffer.push(char);
        Ok(())
    }
    fn set_next_char(&mut self) {
        self.data.char = self.data.input.next();
    }
    fn parse(&mut self, text: &'a String) -> Result<Data, String> {
        self.data.input = text.chars();
        self.run()?;
        Ok(self.data.clone())
    }
}

impl<'a> FindBodyDelimiterTransitions for CsvParser<'a> {
    fn impossible(&mut self) {}
    fn found_new_line(&mut self) -> Result<(), String> {
        let last_row = self.get_last_row()?;
        if last_row.len() != self.data.column_names.len() {
            return Err("Row length does not match column length".to_string());
        }
        let csv = CSVRow::new(self.data.column_names.clone(), vec![String::new()]);
        self.data.rows.push(csv);
        self.set_next_char();
        Ok(())
    }
    fn found_else(&mut self) -> Result<(), String> {
        self.store_char_in_field_buffer()?;
        self.set_next_char();
        Ok(())
    }
    fn found_delimiter(&mut self) -> Result<(), String> {
        self.push_field_to_row()?;
        self.data.field_buffer = String::new();
        self.set_next_char();
        Ok(())
    }
    fn empty(&mut self) -> Result<(), String> {
        self.push_field_to_row()?;
        let last_row = self.get_last_row()?;
        if last_row.len() != self.data.column_names.len() {
            return Err("Row length does not match column length".to_string());
        }
        self.data.field_buffer = String::new();
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
        self.data.column_names = vec![String::new()];
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
        let char = self.data.char.ok_or("char cannot disappear".to_string())?;
        let last_column = self.get_last_column()?;
        last_column.push(char);
        self.set_next_char();
        Ok(())
    }
    fn found_delimiter(&mut self) -> Result<(), String> {
        self.data.column_names.push("".to_string());
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
        let csv = CSVRow::new(self.data.column_names.clone(), vec![]);
        self.data.rows.push(csv);
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
            None => return FindHeaderRightQuoteEvents::Impossible,
        };
        let quote = match self.data.quote {
            Some(c) => c,
            None => return FindHeaderRightQuoteEvents::Impossible,
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
            None => return FindBodyRightQuoteEvents::Impossible,
        };
        let quote = match self.data.quote {
            Some(c) => c,
            None => return FindBodyRightQuoteEvents::Impossible,
        };
        match char {
            c if c == quote => FindBodyRightQuoteEvents::FoundRightQuote,
            '\n' => FindBodyRightQuoteEvents::Impossible,
            _ => FindBodyRightQuoteEvents::FoundElse,
        }
    }
}

fn main() {
    let mut csv_parser = CsvParser::new(',');
    let text = "'a',\"b\",c'b'\n1,2,3".to_string();
    println!("text: {:?}", text);
    let result = csv_parser.parse(&text);
    match result {
        Ok(data) => data
            .rows
            .iter()
            .for_each(|row| println!("column cb: {:?}", row.get("cb".to_string()))),
        Err(e) => println!("Error: {:?}", e),
    }
}

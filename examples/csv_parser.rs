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

#[derive(Debug, Clone, PartialEq)]
struct Data {
    index: usize,
    char: Option<char>,
    quote: Option<char>,
    text: String,
    delimiter: char,
    column_names: Vec<String>,
    field_buffer: String,
    rows: Vec<CSVRow>,
}

state_machine!(
    CSVParser(Data);
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

impl CSVParser {
    fn new(delimiter: char) -> Self {
        CSVParser {
            data: Data {
                field_buffer: String::new(),
                index: 0,
                char: None,
                delimiter,
                quote: None,
                text: String::new(),
                column_names: Vec::new(),
                rows: Vec::new(),
            },
            state: State::Start,
        }
    }
    fn push_field_to_row(&mut self) -> Result<(), String> {
        let field = self.data.field_buffer.clone();
        self.data.field_buffer = String::new();
        self.last_row()?.push(field);
        Ok(())
    }
    fn last_row(&mut self) -> Result<&mut CSVRow, String> {
        self.data
            .rows
            .last_mut()
            .ok_or("last_row is undefined, impossible".to_string())
    }
    fn last_column(&mut self) -> Result<&mut String, String> {
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
    fn parse(&mut self, text: String) -> Result<Data, String> {
        self.data.text = text;
        self.run()?;
        Ok(self.data.clone())
    }
    fn next_char(&mut self) {
        self.data.index += 1;
        self.data.char = self.data.text.chars().nth(self.data.index);
    }
}

impl Transitions for CSVParser {
    fn find_body_delimiter_impossible(&mut self) {}
    fn find_body_right_quote_impossible(&mut self) {}
    fn find_header_delimiter_impossible(&mut self) {}
    fn find_header_right_quote_impossible(&mut self) {}
    fn start_impossible(&mut self) {}
    fn start_begin(&mut self) -> Result<(), String> {
        self.data.char = self.data.text.chars().nth(self.data.index);
        self.data.column_names = vec![String::new()];
        Ok(())
    }
    fn find_header_delimiter_found_else(&mut self) -> Result<(), String> {
        let char = self.data.char.ok_or("char cannot disappear".to_string())?;
        let last_column = self.last_column()?;
        last_column.push(char);
        self.next_char();
        Ok(())
    }
    fn find_header_delimiter_found_delimiter(&mut self) -> Result<(), String> {
        self.data.column_names.push("".to_string());
        self.next_char();
        Ok(())
    }
    fn find_header_delimiter_empty(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn find_header_delimiter_found_left_quote(&mut self) -> Result<(), String> {
        self.data.quote = self.data.char;
        self.next_char();
        Ok(())
    }
    fn find_header_right_quote_found_else(&mut self) -> Result<(), String> {
        self.find_header_delimiter_found_else()
    }
    fn find_header_right_quote_found_right_quote(&mut self) -> Result<(), String> {
        self.data.quote = None;
        self.next_char();
        Ok(())
    }
    fn find_header_delimiter_found_new_line(&mut self) -> Result<(), String> {
        let csv = CSVRow::new(self.data.column_names.clone(), vec![]);
        self.data.rows.push(csv);
        self.next_char();
        Ok(())
    }
    fn find_body_delimiter_found_new_line(&mut self) -> Result<(), String> {
        match self.data.rows.last() {
            Some(row) => {
                println!("row: {:?}", row.len());
                if row.len() != self.data.column_names.len() {
                    return Err("Row length does not match column length".to_string());
                }
            }
            None => return Err("No rows found".to_string()),
        }
        let csv = CSVRow::new(self.data.column_names.clone(), vec![String::new()]);
        self.data.rows.push(csv);
        self.next_char();
        Ok(())
    }
    fn find_body_delimiter_found_else(&mut self) -> Result<(), String> {
        self.store_char_in_field_buffer()?;
        self.next_char();
        Ok(())
    }
    fn find_body_delimiter_found_delimiter(&mut self) -> Result<(), String> {
        self.push_field_to_row()?;
        self.data.field_buffer = String::new();
        self.next_char();
        Ok(())
    }
    fn find_body_delimiter_empty(&mut self) -> Result<(), String> {
        self.push_field_to_row()?;
        let last_row = self.last_row()?;
        if last_row.len() != self.data.column_names.len() {
            return Err("Row length does not match column length".to_string());
        }
        self.data.field_buffer = String::new();
        Ok(())
    }
    fn find_body_delimiter_found_left_quote(&mut self) -> Result<(), String> {
        self.data.quote = self.data.char;
        self.next_char();
        Ok(())
    }
    fn find_body_right_quote_found_right_quote(&mut self) -> Result<(), String> {
        self.data.quote = None;
        self.next_char();
        Ok(())
    }
    fn find_body_right_quote_found_else(&mut self) -> Result<(), String> {
        self.store_char_in_field_buffer()?;
        self.next_char();
        Ok(())
    }
}

impl StateActions for CSVParser {
    fn run_start(&self) -> StartEvents {
        StartEvents::Begin
    }
    fn run_find_header_delimiter(&self) -> FindHeaderDelimiterEvents {
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
    fn run_find_header_right_quote(&self) -> FindHeaderRightQuoteEvents {
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
    fn run_find_body_delimiter(&self) -> FindBodyDelimiterEvents {
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
    fn run_find_body_right_quote(&self) -> FindBodyRightQuoteEvents {
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
    let mut csv_parser = CSVParser::new(',');
    let text = "'a',\"b\",c'b'\n1,2,3".to_string();
    println!("text: {:?}", text);
    let result = csv_parser.parse(text);
    match result {
        Ok(data) => data
            .rows
            .iter()
            .for_each(|row| println!("column cb: {:?}", row.get("cb".to_string()))),
        Err(e) => println!("Error: {:?}", e),
    }
}

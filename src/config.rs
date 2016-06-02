use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};
use std::str;

pub fn read_properties<R: Read>(r: BufReader<R>) -> HashMap<String, String> {
    let mut properties = HashMap::new();
    let mut name: Option<String> = None;
    let mut value = "".to_string();
    for line in r.lines().map(|l| l.unwrap()) {
        let line = line.trim_left().trim_right();
        if line.len() == 0 {
            continue;
        }
        
        match name {
            Some(_) => {
                let trimmed = line.trim_left().trim_right();
                if trimmed.len() > 0 {
                    value = value +
                        if trimmed.as_bytes()[trimmed.len() - 1] == b';' {
                            // Remove semicolon
                            str::from_utf8(&trimmed.as_bytes()[0..trimmed.len() - 1]).unwrap()
                        } else {
                            trimmed
                        };
                    value = value + "\n";
                }
            },
            None => {
                // Coded for clarity. Could iterate, enumerate, and use if statements to avoid
                // Vec and String heap allocations.
                let parts: Vec<String> = line.split("=")
                                             .map(|s| s.trim_left().trim_right().to_string())
                                             .collect();
                name = Some(parts[0].clone());
                if parts[1].len() > 0 {
                    value =
                        if parts[1].as_bytes()[parts[1].len() - 1] == b';' {
                            // Remove semicolon
                            str::from_utf8(&parts[1].as_bytes()[0..parts[1].len() - 1]).unwrap().to_string()
                        } else {
                            parts[1].clone() + "\n"
                        };
                }
            },
        }
        
        if line.as_bytes()[line.len() - 1] == b';' {
            properties.insert(name.take().unwrap(), value.clone());
            value.clear();
        }
    }
    
    properties
}

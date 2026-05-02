use std::io::{BufRead, BufReader, Read, Write};

use crate::Closure;

   pub fn write_cmd<T : Read + Write>(
    stream : &mut T
    ,cmd: &[u8]
) -> Result<(), Box<dyn std::error::Error>> {
        let sending = stream.write_all(cmd)?;
        Ok(sending)
    }

    pub fn read_response<T: Read + Write>(
        stream : &mut T,
        closure: Option<&Closure>,
        response_result: &mut Vec<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut reader = BufReader::new(stream);
        loop {
            let mut response = String::new();
            match reader.read_line(&mut response) {
                Ok(0) => break,
                Ok(_) => {
                    let is_last = response.as_bytes().get(3) == Some(&b' ');
                    // println!("{}", response);
                    if let Some(closure) = closure {
                        closure(response_result, response);
                    }

                    if is_last {
                        break;
                    }

                }
                Err(e) => return Err(Box::new(e)),
            }
        }
        Ok(())
    }
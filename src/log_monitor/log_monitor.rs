use std::collections::VecDeque;
use std::{
    fs::File,
    io::{self, prelude::*, Error, ErrorKind},
    rc::Rc,
};

pub struct LogReader {
    path: String,
    reader: io::BufReader<File>,
    buf: Rc<String>,
    current_line: usize,
}

fn new_buf() -> Rc<String> {
    Rc::new(String::with_capacity(1024)) // Tweakable capacity
}

impl LogReader {
    /// Create a new LogReader
    /// This allows you to monitor a log that is being
    /// actively written to by another thread/process
    /// There is an iterator to read all the lines which
    /// will end once EOF is reached.
    /// But you can call that again periodically as more
    /// lines are written to the log file.
    /// There is also the ability to read a range of line numbers
    /// Generally, just calling read() will get all the lines
    /// from the beginning and subsequent calls will get all
    /// lines that have been written to the file in the interim
    pub fn new(path: &str) -> io::Result<Self> {
        let file = File::open(path)?;
        let reader = io::BufReader::new(file);
        let buf = new_buf();

        Ok(Self { reader, buf, path: path.to_owned(), current_line: 0 })
    }

    pub fn reopen(&mut self) -> io::Result<()> {
        let file = File::open(&self.path)?;
        self.reader = io::BufReader::new(file);
        self.current_line = 0;
        Ok(())
    }

    /// The first call to read() will get all the lines
    /// from the beginning and subsequent calls will get all
    /// lines that have been written to the file in the interim
    pub fn read(&mut self) ->  io::Result<String> {
        let mut out = String::new();
        loop {
            if let Ok(Some(s)) = self.next_line() {
                out += &s;
                // out.push_str("\n");
            } else {
                break;
            }
        }
        Ok(out)
    }

    /// Read no more than n number of lines (at a time)
    /// or to the current end of file
    pub fn read_n_lines(&mut self, n: usize) ->  io::Result<String> {
        let mut out = String::new();
        let mut i = 0;
        loop {
            if let Ok(Some(s)) = self.next_line() {
                out += &s;
                i += 1;
                if i >= n {
                    break;
                }
                // out.push_str("\n");
            } else {
                break;
            }
        }
        Ok(out)
    }

    /// Read the next line
    pub fn next_line(&mut self) ->  io::Result<Option<String>> {
        let line = self.next();
        if let Some(Ok(s)) = line {
            Ok(Some(s.to_string()))
        } else {
            Ok(None)
        }
    }

    /// Read up to the last n lines in the log file
    /// This will not read before the current line unless force=true
    /// 
    /// force : false
    /// If we have already read up to line 10
    /// and there are 12 lines, read_last_n_lines(3)
    /// will only read the last 2 lines.
    /// 
    /// force : true
    /// Reopen the file and read from the beginning again if
    /// we have already read past the the nth last line.
    /// so read_last_n_lines(3) will read the last 3 lines.
    pub fn read_last_n_lines(&mut self, n: usize, force: bool) ->  io::Result<String> {
        let mut cache: VecDeque<String> = VecDeque::with_capacity(n as usize);
        let mut i = 0;
        let prev_current_line = self.current_line;
        loop {
            if let Ok(Some(s)) = self.next_line() {
                i += 1;
                if cache.len() == n {
                    cache.pop_front();
                }
                cache.push_back(s.to_string());
                if i == n {
                    break;
                }
            } else {
                break;
            }
        }
        // we got all we needed, or have available
        if i == n || ! force || prev_current_line == 0 {
            let mut out = String::new();
            for line in cache {
                out.push_str(&line);
            }
            Ok(out)

        // reopen the file and try again
        } else {
            self.reopen()?;
            return self.read_last_n_lines(n, false)
        }
    }


    pub fn read_range_from_end(&mut self, from: isize, to: isize, force: bool) ->  io::Result<String> {
        if from == 0 {
            if to != -1 {
                return Err(Error::new(ErrorKind::Other, "invalid from to range"));
            }
            // from from current position to EOF
            return self.read()
        }

        if to >= 0 {
            return Err(Error::new(ErrorKind::Other, "invalid from to range_from_end"));
        }

        // from is positive so we can compare to current_line
        // Limitation: 
        // if from is positive only -1 for to is allowed for the moment
        if from > 0 {
            let from = from as usize;
            if to == -1{
                if from < self.current_line {
                    self.reopen()?;
                }
                self.skip_n_lines(from - 1);
                self.read()
            } else {
                // e.g.from:50 to:-10
                // get line 
                self.skip_to_end(); // so we know line count
                let mut last_line_to_read = self.current_line as isize;
                last_line_to_read += to;
                let num_lines = last_line_to_read - (from as isize) + 2;
                //if last_line_to_read <= 0 || num_lines =< 0 {
                if num_lines <= 0 {
                    // println!("total lines:{} from:{}", last_line_to_read, from);
                    return Ok(String::from(""));
                }
                // println!("reading from:{} to:{}", from, last_line_to_read);
                self.reopen()?;
                self.skip_n_lines(from - 1);
                self.read_n_lines(num_lines as usize)
            }
        } else {
            let last_n_lines = -from +to +1;  // -(-3) -1 +1 = last 3 lines
            if last_n_lines <= 0 {
                return Err(Error::new(ErrorKind::Other, "invalid from to range"));
            }
            self.read_last_n_lines(last_n_lines as usize, force)
        }
        
    }

    /// Read to end of file. Useful to get the total line count 
    #[allow(dead_code)]
    pub fn current_line(&mut self) -> usize {
        self.current_line
    }

    /// Read to end of file. Useful to get the total line count 
    pub fn skip_to_end(&mut self) {
        loop {
            if let Ok(Some(_)) = self.next_line() {
            } else {
                break;
            }
        }
    }

    /// Skip over n lines
    pub fn skip_n_lines(&mut self, n: usize) {
        for _ in 0..n {
            let line = self.next();
            if let Some(Ok(_)) = line {
                continue;
            } else {
                break
            }
        }
    }

    /// Read from one line number up to another (inclusive)
    /// Read from=1 to=4 will read lines 1, 2, 3, 4
    /// If there are only two lines you will just get those two
    /// lines.
    /// If from=0 it means to continue reading from last line
    /// we read in a previous call to the LogReader.
    /// 
    /// Reading backwards
    /// Read from=-3 to=-1 means to read the last 3 lines
    /// Line number in a six line file
    ///  1   2   3   4   5   6 
    /// -6  -5  -4  -3  -2  -1
    /// 
    /// read_range(4,-2) willl read lines 4, 5
    /// read_range(4,-1) willl read lines 4, 5, 6
    /// read_range(-3,-1) willl read lines 4, 5, 6
    /// read_range(-1,-1) willl read lines 6
    /// read_range(1,1) willl read lines 1
    /// read_range(1,-1) willl read all lines
    pub fn read_range(&mut self, mut from: isize, to: isize, force: bool) ->  io::Result<String> {
        if from == 0 {
            from = self.current_line as isize;
        }
        if to < 1 {
            return self.read_range_from_end(from, to, force)
        }
        if from >= 0 {
            from -= 1;  // read inclusive
        }
        if from > to {
            return Err(Error::new(ErrorKind::Other, "line from > to"));
        }
        let from = from as usize;
        let to = to as usize;
        if from < self.current_line {
            if ! force {
                return Ok(String::new())
            }
            // println!("need to reopen the file");
            self.reopen()?;
        }
        let skip = from - self.current_line;
        self.skip_n_lines(skip);
        // for _ in 1..skip {
        //     let line = self.next();
        //     if let Some(Ok(_)) = line {
        //         continue;
        //     } else {
        //         return Ok(String::new());
        //     }
        // }
        let num_lines_to_read = to - from;
        let mut out = String::new();
        for _ in 0..num_lines_to_read {
            let line = self.next();
            if let Some(Ok(s)) = line {
                out += &s;
                // out.push_str("\n");
            } else {
                break;
            }
        }
        Ok(out)
    }
}

impl Iterator for LogReader {
    type Item = io::Result<Rc<String>>;

    /// Read the next line from the log file
    /// If there is no lines available this will return None
    fn next(&mut self) -> Option<Self::Item> {
        let buf = match Rc::get_mut(&mut self.buf) {
            Some(buf) => {
                buf.clear();
                buf
            }
            None => {
                self.buf = new_buf();
                Rc::make_mut(&mut self.buf)
            }
        };

        self.reader
            .read_line(buf)
            .map(|u| 
                if u == 0 { 
                    None 
                } else { 
                    self.current_line += 1;
                    // println!("current_line:{}", self.current_line);
                    Some(Rc::clone(&self.buf)) 
                }
            )
            .transpose()
    }
}
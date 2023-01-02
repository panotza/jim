pub struct Backend {
    lines: Vec<String>,
}

impl Backend {
    pub fn new(data: String) -> Self {
        Backend {
            lines: data
                .split('\n')
                .map(|line| {
                    let l = line.len();
                    if l > 0 && line.as_bytes()[l - 1] == b'\r' {
                        String::from(&line[0..l - 1])
                    } else {
                        String::from(line)
                    }
                })
                .collect(),
        }
    }

    pub fn get_row(&self, i: usize) -> Option<&str> {
        self.lines.get(i).map(|x| x.as_str())
    }

    pub fn row_length(&self) -> usize {
        self.lines.len()
    }

    pub fn insert(&mut self, l: usize, i: usize, s: char) {
        if let Some(x) = self.lines.get(l) {
            // prepend
            if i == 0 || x.is_empty() {
                self.lines[l] = format!("{}{}", s, x);
                return;
            }

            // append
            if i > x.len() - 1 {
                self.lines[l] = format!("{}{}", x, s);
                return;
            }

            // insert
            let (lhs, rhs) = x.split_at(i);
            self.lines[l] = format!("{}{}{}", lhs, s, rhs);
        }
    }
}

use regex::Regex;

pub struct Input {
    pub values: Vec<f32>,
    prefixes: Vec<String>,
    postfixes: Vec<String>,
    titles: Vec<String>,
    pub time: bool,
    pub overlay: String, //for testing
}

impl Input {
    pub fn new(string: &str, time: bool) -> Self {
        let mut s = Self {
            values: vec![],
            prefixes: vec![],
            postfixes: vec![],
            titles: vec![],
            time,
            overlay: "".to_owned(),
        };

        let lines: Vec<_> = string.split(';').collect::<Vec<&str>>();

        //values
        if !lines.is_empty() {
            let vals = lines[0].split(',').collect::<Vec<&str>>();

            for v in vals {
                //split by text to get pre/postfixes.
                //assumes numbers to be congruent with each other
                let re = Regex::new(r"(.*?)(\-?[0-9]+\.?[0-9]*)(.*)").expect("invalid regex");
                let matches = re.captures(v);
                if let Some(mat) = matches {
                    let pre = &mat[1];
                    let num = &mat[2];
                    let post = &mat[3];

                    if let Ok(val) = num.parse::<f32>() {
                        s.prefixes.push(pre.to_owned());
                        s.values.push(val);
                        s.postfixes.push(post.to_owned());
                    }
                }
            }
        }
        //titles
        if lines.len() >= 2 {
            let titles = lines[1].split(',').map(|x| x.to_owned()).collect::<Vec<String>>();
            s.titles.splice(.., titles);
        }

        s
    }

    pub fn get_string_at(&self, index: usize) -> String {
        let mut string = String::new();
        if self.prefixes.len() > index {
            string.push_str(&self.prefixes[index]);
        }
        if self.values.len() > index {
            string.push_str(&format!("{}", self.values[index]));
        }
        if self.postfixes.len() > index {
            string.push_str(&self.postfixes[index]);
        }

        string
    }
    pub fn get_title_at(&self, index: usize) -> String {
        if self.titles.len() > index {
            self.titles[index].to_owned()
        } else {
            "".to_string()
        }
    }
}

#[macro_use]
extern crate serde_derive;

extern crate chrono;
extern crate dirs;
extern crate serde_yaml;

use std::io::Write;

const DATA_FILE: &str = ".tikdata";

fn data_file() -> String {
    match dirs::home_dir() {
        Some(mut path) => {
            path.push(DATA_FILE);
            path.to_string_lossy().to_string()
        },
        None => DATA_FILE.to_string(),
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Tik {
    days: Vec<Day>,
}

impl Tik {

    pub fn new() -> Tik {
        Tik{days: vec![]}
    }

    pub fn load() -> Tik {
        match std::fs::read_to_string(data_file()) {
            Ok(data) => Tik::from_yaml(data),
            Err(_) => Tik::new(),
        }
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        match std::fs::OpenOptions::new().create(true).write(true).truncate(true).open(data_file()) {
            Ok(mut file) => file.write_all(self.to_yaml().as_bytes()),
            Err(e) => Err(e),
        }
    }

    pub fn from_yaml(data: String) -> Tik {
        match serde_yaml::from_str(&data) {
            Ok(yaml) => yaml,
            Err(_) => Tik::new(),
        }
    }

    pub fn to_yaml(&self) -> String {
        match serde_yaml::to_string(&self) {
            Ok(yaml) => yaml,
            Err(_) => "---".to_string(),
        }
    }

    pub fn add_entry(&mut self, date: String, entry: Entry) {
        let mut found = false;
        for mut day in self.days.iter_mut() {
            if date == day.date {
                found = true;
                day.add_entry(entry.clone());
            }
        }
        if !found {
            self.days.push(Day{date: date, entries: vec![entry]});
        }
    }
}

impl Day {
    pub fn add_entry(&mut self, entry: Entry) {
        self.entries.push(entry);
    }
}

impl Entry {
    pub fn to_string(&self) -> String {
        format!("{} -> {}", self.time, self.subject)
    }
}

#[test]
fn test_tik() {
    let mut empty_tik = Tik::from_yaml("".to_string());
    assert_eq!(empty_tik.days.len(), 0);
    empty_tik.add_entry("2018-01-01".to_string(), Entry{time: "00:00:00".to_string(), subject: "Potatoes".to_string()});

    assert_eq!(empty_tik.days.len(), 1);
    assert_eq!(empty_tik.days[0].date, "2018-01-01".to_string());
    assert_eq!(empty_tik.days[0].entries.len(), 1);
    assert_eq!(empty_tik.days[0].entries[0].time, "00:00:00".to_string());
    assert_eq!(empty_tik.days[0].entries[0].subject, "Potatoes".to_string());

    empty_tik.add_entry("2018-01-01".to_string(), Entry{time: "01:00:00".to_string(), subject: "Tomatoes".to_string()});

    assert_eq!(empty_tik.days.len(), 1);
    assert_eq!(empty_tik.days[0].date, "2018-01-01".to_string());
    assert_eq!(empty_tik.days[0].entries.len(), 2);
    assert_eq!(empty_tik.days[0].entries[1].time, "01:00:00".to_string());
    assert_eq!(empty_tik.days[0].entries[1].subject, "Tomatoes".to_string());
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Day {
    date: String,
    entries: Vec<Entry>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Entry {
    time: String,
    subject: String,
}

fn main() {
    let local = chrono::Local::now();
    let time_string = local.time().format("%H:%M:%S").to_string();
    let date_string = local.date().format("%Y-%m-%d").to_string();

    let args: Vec<String> = std::env::args().collect();

    if args.len() == 1 {
        let tikdata = Tik::load();
        println!("{}", tikdata.to_yaml());
    } else {
        let data = args[1..].join(" ");
        let entry = Entry{time: time_string, subject: data};

        let mut tikdata = Tik::load();
        tikdata.add_entry(date_string, entry.clone());
        match tikdata.save() {
            Ok(_) => println!("{}", entry.to_string()),
            Err(e) => println!("{}", e),
        }

    }
}

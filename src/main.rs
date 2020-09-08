#[macro_use]
extern crate serde_derive;

extern crate chrono;
extern crate dirs;
extern crate serde_yaml;

use std::io::Write;
use std::str::FromStr;

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

    pub fn count(&self, date: String) -> chrono::Duration {
        match self.days.iter().find(|day| day.date == date) {
            None => chrono::Duration::zero(),
            Some(day) => {
                day.get_work_sessions().iter().fold(
                    chrono::Duration::zero(),
                    |acc, session| {
                        if session.is_closed() {
                            acc + (session.end.unwrap() - session.start.unwrap())
                        } else {
                            acc
                        }
                    })
            },
        }
    }
}

#[derive(Debug)]
struct Session {
    start: Option<chrono::naive::NaiveTime>,
    end: Option<chrono::naive::NaiveTime>,
}

impl Session {

    fn new(time: String) -> Session {
        Session {
            start: chrono::naive::NaiveTime::parse_from_str(&time,"%H:%M:%S").ok(),
            end: None,
        }
    }

    fn close(&mut self, time: String) {
        if !self.is_closed() {
            self.end = chrono::naive::NaiveTime::parse_from_str(&time,"%H:%M:%S").ok();
        }
    }

    fn is_closed(&self) -> bool {
        self.start.is_some() && self.end.is_some()
    }

}

impl Day {
    pub fn add_entry(&mut self, entry: Entry) {
        self.entries.push(entry);
    }

    fn get_work_sessions(&self) -> Vec<Session> {
        let mut sessions: Vec<Session> = Vec::new();
        for entry in self.entries.iter() {
            match entry.subject.as_str() {
                "stop" => {
                    match sessions.iter_mut().last() {
                        None => (),
                        Some(session) => {
                            session.close(entry.time.clone())
                        }
                    }
                },
                _ => {
                    match sessions.iter().last() {
                        None => sessions.push(
                            Session::new(entry.time.clone())),
                        Some(session) if session.is_closed() => {
                            sessions.push(
                                Session::new(entry.time.clone()))
                        },
                        _ => (),
                    }
                }
            }
        }
        sessions
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

#[test]
fn test_count_zero_when_empty() {
    let mut empty_tik = Tik::from_yaml("".to_string());
    assert_eq!(empty_tik.count("2018-01-03".to_string()), chrono::Duration::zero());
}

#[test]
fn test_count_hours_worked_in_one_session() {
    let mut empty_tik = Tik::from_yaml("".to_string());
    empty_tik.add_entry("2018-01-03".to_string(), Entry{time: "01:00:00".to_string(), subject: "Tomatoes".to_string()});
    empty_tik.add_entry("2018-01-03".to_string(), Entry{time: "03:00:00".to_string(), subject: "stop".to_string()});
    assert_eq!(empty_tik.count("2018-01-03".to_string()), chrono::Duration::hours(2));
}

#[test]
fn test_count_dont_count_unclosed_session() {
    let mut empty_tik = Tik::from_yaml("".to_string());
    empty_tik.add_entry("2018-01-03".to_string(), Entry{time: "01:00:00".to_string(), subject: "Tomatoes".to_string()});
    assert_eq!(empty_tik.count("2018-01-03".to_string()), chrono::Duration::hours(0));
}

#[test]
fn test_count_dont_count_unopened_session() {
    let mut empty_tik = Tik::from_yaml("".to_string());
    empty_tik.add_entry("2018-01-03".to_string(), Entry{time: "01:00:00".to_string(), subject: "Tomatoes".to_string()});
    empty_tik.add_entry("2018-01-03".to_string(), Entry{time: "05:00:00".to_string(), subject: "stop".to_string()});
    empty_tik.add_entry("2018-01-03".to_string(), Entry{time: "07:00:00".to_string(), subject: "stop".to_string()});
    assert_eq!(empty_tik.count("2018-01-03".to_string()), chrono::Duration::hours(4));
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
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 1 {
        let tikdata = Tik::load();
        println!("{}", tikdata.to_yaml());
    } else {
        let mut tikdata = Tik::load();
        let local = chrono::Local::now();
        let time_string = local.time().format("%H:%M:%S").to_string();
        let date_string = local.date().format("%Y-%m-%d").to_string();
        match args[1].as_str() {
            "count" => {
                let entry = Entry{time: time_string, subject: "stop".to_string()};
                tikdata.add_entry(date_string.clone(), entry.clone());
                let count = chrono::naive::NaiveTime::from_hms(0,0,0) + tikdata.count(date_string);
                println!("{}", count.format("%H:%M:%S"))
            },
            _ => {
                let data = args[1..].join(" ");
                let entry = Entry{time: time_string, subject: data};
                tikdata.add_entry(date_string, entry.clone());
                match tikdata.save() {
                    Ok(_) => println!("{}", entry.to_string()),
                    Err(e) => println!("{}", e),
                }
            }
        }
    }
}

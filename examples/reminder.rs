extern crate chrono;
extern crate object_space;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use std::io::{stdin, stdout, Write};
use std::process::exit;
use std::thread;
use std::sync::Arc;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::fmt;
use std::time::Duration;

use chrono::{DateTime, Duration as ChronoDuration, NaiveDateTime, Utc};
use object_space::object_space::{ObjectSpace, ObjectSpaceKey, ObjectSpaceRange, TreeObjectSpace};

fn main() {
    let store = ReminderStore::new();
    run(store);
}

fn run(store: ReminderStore) {
    let arc = Arc::new(store);
    let main_clone = arc.clone();
    let thread1 = thread::spawn(move || main_clone.main_loop());
    let thread2 = thread::spawn(move || arc.clone().check_reminder());
    thread1.join().unwrap();
    thread2.join().unwrap();
}

#[derive(Serialize, Deserialize, Debug)]
struct Reminder {
    id: isize,
    time: i64,
    content: String,
}

struct ReminderStore {
    space: TreeObjectSpace,
    counter: AtomicIsize,
}

impl ReminderStore {
    fn check_reminder(&self) {
        loop {
            thread::sleep(Duration::new(60, 0));
            let now = Utc::now();
            for r in self.get_reminder_between_time(now, now + ChronoDuration::minutes(1)) {
                println!();
                println!("{}", r);
                print!(">>> ");
                stdout().flush().unwrap();
            }
        }
    }

    fn main_loop(&self) {
        loop {
            print!(">>> ");
            let _ = stdout().flush();
            let mut input = String::new();
            match stdin().read_line(&mut input) {
                Ok(_) => {
                    let mut input_split = input.split_whitespace();
                    match input_split.next() {
                        Some("quit") => {
                            println!("Exiting");
                            exit(0)
                        }
                        Some("add") => self.request_reminder_info(),
                        Some("complete") => {
                            let id = input_split.next().and_then(|s| s.parse::<isize>().ok());
                            match id {
                                Some(n) => {
                                    self.complete_reminder(n);
                                    ()
                                }
                                None => println!("Please provide reminder id"),
                            }
                        }
                        Some("all") => for r in self.get_all_todo_reminders() {
                            println!("{}", r);
                        },
                        Some("next") => match self.get_next_reminder() {
                            Some(r) => println!("{}", r),
                            None => println!("There is no reminder here"),
                        },
                        Some("outdated") => for r in self.get_all_outdated_reminders() {
                            println!("{}", r);
                        },
                        _ => println!("Unrecognizable command: {}", &input),
                    }
                }
                Err(error) => println!("error: {}", error),
            }
        }
    }

    fn request_reminder_info(&self) {
        print!("Reminder content: ");
        let _ = stdout().flush();
        let mut content = String::new();
        match stdin().read_line(&mut content) {
            Err(_) => {
                println!("Cannot read input");
                return;
            }
            Ok(_) => (),
        }

        print!("Minutes to remind: ");
        let _ = stdout().flush();
        let mut time_str = String::new();
        match stdin().read_line(&mut time_str) {
            Err(_) => {
                println!("Cannot read input");
                return;
            }
            Ok(_) => (),
        }
        let id = time_str.trim().parse::<i64>();
        match id {
            Ok(n) => {
                self.add_reminder(
                    Utc::now() + ChronoDuration::minutes(n),
                    content.trim().to_owned(),
                );
            }
            Err(_) => println!("Please provide numeric minutes to remind"),
        }
    }

    fn new() -> ReminderStore {
        ReminderStore {
            space: TreeObjectSpace::new(),
            counter: AtomicIsize::new(0),
        }
    }

    fn add_reminder(&self, time: DateTime<Utc>, content: String) {
        let id = self.counter.fetch_add(1, Ordering::Relaxed);
        self.space.write(Reminder {
            id: id,
            time: time.timestamp(),
            content: content,
        });
    }

    fn get_reminder_until_time<'a>(
        &'a self,
        time: DateTime<Utc>,
    ) -> Box<Iterator<Item = Reminder> + 'a> {
        self.space
            .read_all_range::<Reminder, _>("time", Utc::now().timestamp()..time.timestamp())
    }

    fn get_reminder_between_time<'a>(
        &'a self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Box<Iterator<Item = Reminder> + 'a> {
        self.space
            .read_all_range::<Reminder, _>("time", start_time.timestamp()..end_time.timestamp())
    }

    fn get_all_todo_reminders<'a>(&'a self) -> Box<Iterator<Item = Reminder> + 'a> {
        self.space
            .read_all_range::<Reminder, _>("time", Utc::now().timestamp()..)
    }

    fn get_all_outdated_reminders<'a>(&'a self) -> Box<Iterator<Item = Reminder> + 'a> {
        self.space
            .read_all_range::<Reminder, _>("time", ..Utc::now().timestamp())
    }

    fn complete_reminder(&self, id: isize) -> Option<Reminder> {
        self.space.try_take_key::<Reminder>("id", &(id as i64))
    }

    fn edit_reminder_content(&self, id: isize, content: &str) {
        match self.space.try_take_key::<Reminder>("id", &(id as i64)) {
            Some(Reminder {
                id: _,
                time: rtime,
                content: _,
            }) => self.space.write(Reminder {
                id: id,
                time: rtime,
                content: content.to_owned(),
            }),
            _ => {}
        }
    }

    fn edit_reminder_time(&self, id: isize, time: DateTime<Utc>) {
        match self.space.try_take_key::<Reminder>("id", &(id as i64)) {
            Some(Reminder {
                id: _,
                time: _,
                content: rcontent,
            }) => self.space.write(Reminder {
                id: id,
                time: time.timestamp(),
                content: rcontent,
            }),
            _ => {}
        }
    }

    fn get_next_reminder(&self) -> Option<Reminder> {
        self.space
            .read_all_range::<Reminder, _>("time", Utc::now().timestamp()..)
            .min_by_key(|r| r.time)
    }
}

impl fmt::Display for Reminder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Reminder id: {}, content: {}, remind time: {}",
            self.id,
            self.content,
            DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(self.time, 0), Utc)
        )
    }
}

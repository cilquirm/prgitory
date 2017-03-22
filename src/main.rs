
#[macro_use] extern crate lazy_static;
extern crate slack;
extern crate regex;

use std::collections::VecDeque;
use regex::Regex;

#[derive(Debug)]
struct PullRequestEntry {
    pull_request_number: i64,
    pull_request_url: String,
}

#[derive(Debug)]
enum Command {
    DoNothing,
    AddPullRequestToQueue(PullRequestEntry)
}


struct PrgitoryHandler {
    pull_request_queue: VecDeque<PullRequestEntry>
}

impl PrgitoryHandler {
    fn new() -> Self {
        PrgitoryHandler {
            pull_request_queue: VecDeque::new()
        }

    }

    fn process_message(&mut self, s: String) -> Command {

        lazy_static! {
            static ref PULL_REQUEST_REGEX: Regex = Regex::new(r".*(http[s]?://github.com/[^/]+/[^/]+/pull/([1-9]+)).*").unwrap();
        }

        let captures = PULL_REQUEST_REGEX.captures(&s);

        match captures {
            Some(captures) => Command::AddPullRequestToQueue(PullRequestEntry {
                pull_request_number: String::from(captures.get(2).expect("should have matched pull request number").as_str()).parse::<i64>().unwrap(),
                pull_request_url: String::from(captures.get(1).expect("Should have matched!").as_str()),
            }),
            None => Command::DoNothing,
        }
    }
}

#[allow(unused_variables)]
impl slack::EventHandler for PrgitoryHandler {

    fn on_event(&mut self, cli: &mut slack::RtmClient, event: Result<slack::Event, slack::Error>, raw_json: &str) {

        let text = event.ok().map( |evt|  {
            match evt {
                slack::Event::Message( slack::Message::Standard{ text, .. } ) => text,
                _ => None
            }
        }).unwrap();

        let command = text.map_or(Command::DoNothing, |txt| { self.process_message(txt) });

        match command {
            Command::AddPullRequestToQueue( pull_request_entry ) => self.pull_request_queue.push_back(pull_request_entry),
            Command::DoNothing => {},
        }

    }

    fn on_ping(&mut self, cli: &mut slack::RtmClient) {

        println!("{:#?}", self.pull_request_queue);

    }

    fn on_close(&mut self, cli: &mut slack::RtmClient) {
    }

    fn on_connect(&mut self, cli: &mut slack::RtmClient) {
        println!("on_connect");

    }
}

fn main() {
    println!("Starting prgitory...");

    let args: Vec<String> = std::env::args().collect();

    let api_key = match args.len() {
        0 | 1 => panic!("Please specify api key"),
          x => {
              args[x - 1].clone()
          }
    };

    let mut handler = PrgitoryHandler::new();
    let mut cli = slack::RtmClient::new(&api_key);

    let runner = cli.login_and_run::<PrgitoryHandler>(&mut handler);

    match runner {
        Ok(_) => {}
        Err(err) => panic!("Error: {}", err),
    }

    println!("{}", cli.get_name().unwrap());
    println!("{}", cli.get_team().unwrap().name);
}

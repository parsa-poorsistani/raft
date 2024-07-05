use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum State {
    Leader,
    Follower,
    Candidate,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub term: usize,
    pub command: String,
}

#[derive()]
pub struct AppendEntriesArgs {
    pub term: usize,
    pub leader_id: String,
    pub prev_log_index: usize,
    pub prev_log_term: usize,
    pub entries: Vec<LogEntry>,
    pub leader_commit: usize,
}

#[derive()]
pub struct AppendEntriesReply {
    pub term: usize,
    pub success: bool,
}

#[derive()]
pub struct RequestVoteArgs {
    pub term: usize,
    pub candidate_id: String,
    pub last_log_index: usize,
    pub last_log_term: usize,
}

#[derive()]
pub struct RequestVoteReply {
    pub term: usize,
    pub granted: bool,
}

pub struct RaftNode {
    pub id: String,
    pub peers: Vec<String>,
    pub state: State,
    pub current_term: usize,
    pub voted_for: Option<String>,
    pub log: Vec<LogEntry>,
    pub commit_index: usize, // The index of the highest log entry known to be committed
    pub last_applied: usize, // The index of the highest log entry applied to the state machine
    pub next_index: HashMap<String, usize>, // For each follower, the index of the next log entry to send to that follower
    pub match_index: HashMap<String, usize>, // For each follower, the index of the highest log entry known to be replicated on that follower
}

impl RaftNode {
    pub fn new(id: String, peers: Vec<String>) -> Self {
        Self {
            id,
            state: State::Follower,
            peers,
            current_term: 0,
            voted_for: None,
            log: Vec::new(),
            commit_index: 0,
            last_applied: 0,
            next_index: HashMap::new(),
            match_index: HashMap::new(),
        }
    }

    pub fn handle_append_entries(&mut self, args: AppendEntriesArgs) -> AppendEntriesReply {
        if args.term < self.current_term {
            return AppendEntriesReply {
                term: self.current_term,
                success: false,
            };
        }

        self.current_term = args.term;

        if let Some(log_entry) = self.log.get(args.prev_log_index) {
            if log_entry.term != args.prev_log_term {
                return AppendEntriesReply {
                    term: self.current_term,
                    success: false,
                };
            }
        } else {
            return AppendEntriesReply {
                term: self.current_term,
                success: false,
            };
        }

        self.log.truncate(args.prev_log_index + 1);
        self.log.extend(args.entries);

        if args.leader_commit > self.commit_index {
            self.commit_index = std::cmp::min(args.leader_commit, self.log.len() - 1);
        }

        AppendEntriesReply {
            term: self.current_term,
            success: true,
        }
    }

    pub fn handle_request_vote(&mut self, args: RequestVoteArgs) -> RequestVoteReply {
        if args.term < self.current_term {
            return RequestVoteReply {
                term: self.current_term,
                granted: false,
            };
        }
        if args.term > self.current_term {
            self.current_term = args.term;
            self.voted_for = None;
        }

        let last_log_index = self.log.len() - 1;
        let last_log_term = self.log[last_log_index].term;

        let log_up_to_date = (args.last_log_term > last_log_term)
            || (args.last_log_term == last_log_term && args.last_log_index >= last_log_index);

        if log_up_to_date
            && (self.voted_for.is_none() || self.voted_for == Some(args.candidate_id.clone()))
        {
            self.voted_for = Some(args.candidate_id.clone());
            return RequestVoteReply {
                term: self.current_term,
                granted: true,
            };
        }

        RequestVoteReply {
            term: self.current_term,
            granted: false,
        }
    }
}

pub mod raft {
    use std::collections::HashMap;

    #[derive(Debug, Clone, PartialEq)]
    pub enum State {
        Leader,
        Follower,
        Candidate,
    }

    #[derive(Debug)]
    pub struct RaftNode {
        id: String,
        peers: Vec<String>,
        state: State,
        current_term: usize,
        voted_for: Option<usize>,
        log: Vec<LogEntry>,
        commit_index: usize, // The index of the highest log entry known to be committed, Ex: 5 means all log entries up to index 5 are committed
        last_applied: usize, // The index of the highest log entry applied to the state machine
        next_index: HashMap<String, usize>, // For each follower, the index of the next log entry to send to that follower
        match_index: HashMap<String, usize>, // For each follower, the index of the highest log entry known to be replicated on that follower
    }

    #[derive(Clone, Debug)]
    pub struct LogEntry {
        term: usize,
        command: String,
    }

    pub struct AppendEntriesArgs {
        term: usize,
        leader_id: String,
        prev_log_index: usize,
        prev_log_term: usize,
        entries: Vec<LogEntry>,
        leader_commit: usize,
    }

    pub struct AppendEntriesReply {
        term: usize,
        success: bool,
    }

    pub struct RequestVoteArgs {
        term: usize,
        candidate_id: usize,
        last_log_index: usize,
        last_log_term: usize,
    }

    pub struct RequestVoteReply {
        term: usize,
        granted: bool,
    }

    impl RaftNode {
        pub fn new(id: String, peers: Vec<String>) -> Self {
            // Should set the ID
            Self {
                id: id,
                state: State::Follower,
                peers: peers,
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
                && (self.voted_for.is_none() || self.voted_for == Some(args.candidate_id))
            {
                self.voted_for = Some(args.candidate_id);
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

        pub fn send_heartbeat(&self) {
            let args = AppendEntriesArgs {
                term: self.current_term,
                leader_id: self.id.to_string(),
                prev_log_term: self.log.last().unwrap().term,
                prev_log_index: self.log.len() - 1,
                entries: vec![],
                leader_commit: self.commit_index,
            };

            for peer in &self.peers {
                // RPC
            }
        }
    }
}

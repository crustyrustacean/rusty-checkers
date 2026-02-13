// backend/src/tournament/manager.rs

use checkers_common::tournament::{Match, TournamentPlayer, TournamentState, TournamentView};
use rand::seq::SliceRandom;
use std::time::Instant;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct MatchStartAction {
    pub match_id: Uuid,
    pub game_id: String,
    pub player1_id: Uuid,
    pub player1_name: String,
    pub player2_id: Uuid,
    pub player2_name: String,
}

const TOURNAMENT_TTL: std::time::Duration = std::time::Duration::from_secs(3600);

pub struct TournamentManager {
    code: String,
    state: TournamentState,
    players: Vec<TournamentPlayer>,
    matches: Vec<Match>,
    host_id: Uuid,
    total_rounds: usize,
    finished_at: Option<Instant>,
}

impl TournamentManager {
    pub fn new(code: String, host_name: String) -> Self {
        let host = TournamentPlayer::new(host_name);
        let host_id = host.id;

        Self {
            code,
            state: TournamentState::Lobby,
            players: vec![host],
            matches: Vec::new(),
            host_id,
            total_rounds: 0,
            finished_at: None,
        }
    }

    pub fn host_id(&self) -> Uuid {
        self.host_id
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn set_code(&mut self, code: String) {
        self.code = code;
    }

    pub fn add_player(&mut self, name: String) -> Result<Uuid, String> {
        if self.state != TournamentState::Lobby {
            return Err("Tournament has already started".into());
        }
        let player = TournamentPlayer::new(name);
        let id = player.id;
        self.players.push(player);
        Ok(id)
    }

    pub fn find_player(&self, id: Uuid) -> Option<&TournamentPlayer> {
        self.players.iter().find(|p| p.id == id)
    }

    fn player_name(&self, id: Uuid) -> String {
        self.find_player(id)
            .map(|p| p.name.clone())
            .unwrap_or_else(|| "Unknown".into())
    }

    pub fn start(&mut self) -> Result<Vec<MatchStartAction>, String> {
        if self.state != TournamentState::Lobby {
            return Err("Tournament already started".into());
        }
        if self.players.len() < 2 {
            return Err("Need at least 2 players to start".into());
        }

        self.state = TournamentState::InProgress;

        let mut rng = rand::thread_rng();
        let mut player_ids: Vec<Uuid> = self.players.iter().map(|p| p.id).collect();
        player_ids.shuffle(&mut rng);

        let bracket_size = player_ids.len().next_power_of_two();
        self.total_rounds = (bracket_size as f64).log2() as usize;

        let matches_in_round = bracket_size / 2;
        let mut actions = Vec::new();

        for position in 0..matches_in_round {
            let mut m = Match::new(0, position);

            let p1_idx = position * 2;
            let p2_idx = position * 2 + 1;

            if p1_idx < player_ids.len() {
                m.player1_id = Some(player_ids[p1_idx]);
            }
            if p2_idx < player_ids.len() {
                m.player2_id = Some(player_ids[p2_idx]);
            }

            if m.player1_id.is_some() && m.player2_id.is_none() {
                m.winner_id = m.player1_id;
            } else if m.player2_id.is_some() && m.player1_id.is_none() {
                m.winner_id = m.player2_id;
            }

            if m.is_ready() && !m.is_complete() {
                let game_id = Uuid::new_v4().to_string();
                m.game_id = Some(game_id.clone());

                actions.push(MatchStartAction {
                    match_id: m.id,
                    game_id,
                    player1_id: m.player1_id.unwrap(),
                    player1_name: self.player_name(m.player1_id.unwrap()),
                    player2_id: m.player2_id.unwrap(),
                    player2_name: self.player_name(m.player2_id.unwrap()),
                });
            }

            self.matches.push(m);
        }

        let mut matches_remaining = matches_in_round / 2;
        for round in 1..self.total_rounds {
            for position in 0..matches_remaining {
                self.matches.push(Match::new(round, position));
            }
            matches_remaining /= 2;
        }

        self.propagate_byes(&mut actions);

        Ok(actions)
    }

    fn propagate_byes(&mut self, actions: &mut Vec<MatchStartAction>) {
        // Collect bye winners from round 0
        let round0: Vec<(usize, Uuid)> = self
            .matches
            .iter()
            .filter(|m| m.round == 0 && m.is_complete())
            .map(|m| (m.position, m.winner_id.unwrap()))
            .collect();

        for (position, winner_id) in round0 {
            self.seat_winner_in_next_round(0, position, winner_id, actions);
        }
    }

    fn seat_winner_in_next_round(
        &mut self,
        round: usize,
        position: usize,
        winner_id: Uuid,
        actions: &mut Vec<MatchStartAction>,
    ) {
        let next_round = round + 1;
        if next_round >= self.total_rounds {
            return; // Tournament is complete
        }

        let next_position = position / 2;
        let is_first_slot = position.is_multiple_of(2);

        if let Some(next_match) = self
            .matches
            .iter_mut()
            .find(|m| m.round == next_round && m.position == next_position)
        {
            if is_first_slot {
                next_match.player1_id = Some(winner_id);
            } else {
                next_match.player2_id = Some(winner_id);
            }

            // If both players are now present, create a game
            if next_match.is_ready() && !next_match.is_complete() {
                let game_id = Uuid::new_v4().to_string();
                next_match.game_id = Some(game_id.clone());

                let p1 = next_match.player1_id.unwrap();
                let p2 = next_match.player2_id.unwrap();

                actions.push(MatchStartAction {
                    match_id: next_match.id,
                    game_id,
                    player1_id: p1,
                    player1_name: self.player_name(p1),
                    player2_id: p2,
                    player2_name: self.player_name(p2),
                });
            }
        }
    }

    pub fn advance(
        &mut self,
        match_id: Uuid,
        winner_id: Uuid,
    ) -> Result<Option<MatchStartAction>, String> {
        let (round, position) = {
            let m = self
                .matches
                .iter_mut()
                .find(|m| m.id == match_id)
                .ok_or("Match not found")?;

            if m.player1_id != Some(winner_id) && m.player2_id != Some(winner_id) {
                return Err("Winner is not a participant of this match".into());
            }

            if m.is_complete() {
                return Err("Match already completed".into());
            }

            m.winner_id = Some(winner_id);
            (m.round, m.position)
        };

        if round + 1 >= self.total_rounds {
            self.state = TournamentState::Finished;
            self.finished_at = Some(Instant::now());
            return Ok(None);
        }

        // Seat winner in next round
        let mut actions = Vec::new();
        self.seat_winner_in_next_round(round, position, winner_id, &mut actions);

        Ok(actions.into_iter().next())
    }

    pub fn forfeit(
        &mut self,
        match_id: Uuid,
        forfeiter_id: Uuid,
    ) -> Result<Option<MatchStartAction>, String> {
        let opponent_id = {
            let m = self
                .matches
                .iter()
                .find(|m| m.id == match_id)
                .ok_or("Match not found")?;

            if m.player1_id == Some(forfeiter_id) {
                m.player2_id.ok_or("No opponent to advance")?
            } else if m.player2_id == Some(forfeiter_id) {
                m.player1_id.ok_or("No opponent to advance")?
            } else {
                return Err("Player is not in this match".into());
            }
        };

        self.advance(match_id, opponent_id)
    }

    pub fn find_match_by_game_id(&self, game_id: &str) -> Option<Uuid> {
        self.matches
            .iter()
            .find(|m| m.game_id.as_deref() == Some(game_id))
            .map(|m| m.id)
    }

    pub fn find_match(&self, match_id: Uuid) -> Option<&Match> {
        self.matches.iter().find(|m| m.id == match_id)
    }

    pub fn find_active_match_for_player(&self, player_id: Uuid) -> Option<&Match> {
        self.matches.iter().find(|m| {
            !m.is_complete()
                && m.game_id.is_some()
                && (m.player1_id == Some(player_id) || m.player2_id == Some(player_id))
        })
    }

    pub fn view(&self) -> TournamentView {
        let winner = if self.state == TournamentState::Finished {
            self.matches
                .iter()
                .filter(|m| m.round + 1 == self.total_rounds && m.is_complete())
                .find_map(|m| m.winner_id.and_then(|wid| self.find_player(wid).cloned()))
        } else {
            None
        };

        let current_round = self
            .matches
            .iter()
            .filter(|m| !m.is_complete() && (m.player1_id.is_some() || m.player2_id.is_some()))
            .map(|m| m.round)
            .min()
            .unwrap_or(if self.total_rounds > 0 {
                self.total_rounds - 1
            } else {
                0
            });

        TournamentView {
            code: self.code.clone(),
            state: self.state.clone(),
            players: self.players.clone(),
            matches: self.matches.clone(),
            current_round,
            total_rounds: self.total_rounds,
            winner,
        }
    }

    pub fn disconnect_player(&mut self, player_id: Uuid) {
        if let Some(p) = self.players.iter_mut().find(|p| p.id == player_id) {
            p.connected = false;
        }
    }

    pub fn reconnect_player(&mut self, player_id: Uuid) {
        if let Some(p) = self.players.iter_mut().find(|p| p.id == player_id) {
            p.connected = true;
        }
    }

    pub fn is_stale(&self) -> bool {
        self.finished_at
            .map(|t| t.elapsed() > TOURNAMENT_TTL)
            .unwrap_or(false)
    }

    pub fn state(&self) -> &TournamentState {
        &self.state
    }
}

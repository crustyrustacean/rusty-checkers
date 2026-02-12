// backend/src/tournament/manager.rs

use checkers_common::tournament::{Match, TournamentPlayer, TournamentState, TournamentView};
use rand::seq::SliceRandom;
use std::time::Instant;
use uuid::Uuid;

/// Describes a match that is ready to start and needs a game session created.
#[derive(Debug, Clone)]
pub struct MatchStartAction {
    pub match_id: Uuid,
    pub game_id: String,
    pub player1_id: Uuid,
    pub player1_name: String,
    pub player2_id: Uuid,
    pub player2_name: String,
}

/// Manages the lifecycle of a single-elimination bracket tournament.
///
/// This struct is game-agnostic — it only tracks players, bracket structure,
/// and match outcomes. The actual game sessions are created externally.
/// Time-to-live for finished tournaments before they are cleaned up.
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

    /// Add a player to the tournament lobby.
    ///
    /// Returns the new player's ID, or an error if the tournament has already started.
    pub fn add_player(&mut self, name: String) -> Result<Uuid, String> {
        if self.state != TournamentState::Lobby {
            return Err("Tournament has already started".into());
        }
        let player = TournamentPlayer::new(name);
        let id = player.id;
        self.players.push(player);
        Ok(id)
    }

    /// Look up a player by ID.
    pub fn find_player(&self, id: Uuid) -> Option<&TournamentPlayer> {
        self.players.iter().find(|p| p.id == id)
    }

    /// Look up a player's name by ID.
    fn player_name(&self, id: Uuid) -> String {
        self.find_player(id)
            .map(|p| p.name.clone())
            .unwrap_or_else(|| "Unknown".into())
    }

    /// Start the tournament: shuffle players, generate bracket, handle byes.
    ///
    /// Returns the list of matches that need game sessions created immediately.
    pub fn start(&mut self) -> Result<Vec<MatchStartAction>, String> {
        if self.state != TournamentState::Lobby {
            return Err("Tournament already started".into());
        }
        if self.players.len() < 2 {
            return Err("Need at least 2 players to start".into());
        }

        self.state = TournamentState::InProgress;

        // Shuffle players for random seeding
        let mut rng = rand::thread_rng();
        let mut player_ids: Vec<Uuid> = self.players.iter().map(|p| p.id).collect();
        player_ids.shuffle(&mut rng);

        // Calculate bracket size: next power of 2 >= player count
        let bracket_size = player_ids.len().next_power_of_two();
        self.total_rounds = (bracket_size as f64).log2() as usize;

        // Create round 1 matches
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

            // Handle bye: only one player in this slot
            if m.player1_id.is_some() && m.player2_id.is_none() {
                // Auto-advance player1
                m.winner_id = m.player1_id;
            } else if m.player2_id.is_some() && m.player1_id.is_none() {
                // Auto-advance player2
                m.winner_id = m.player2_id;
            }

            // If both players present, create a game session
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

        // Pre-create placeholder matches for all future rounds
        let mut matches_remaining = matches_in_round / 2;
        for round in 1..self.total_rounds {
            for position in 0..matches_remaining {
                self.matches.push(Match::new(round, position));
            }
            matches_remaining /= 2;
        }

        // Propagate any byes into round 2
        self.propagate_byes(&mut actions);

        Ok(actions)
    }

    /// After initial bracket creation, propagate bye winners into the next round.
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

    /// After a match completes, seat the winner in the correct slot of the next round.
    ///
    /// Returns a `MatchStartAction` if the next-round match now has both players.
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
        let is_first_slot = position % 2 == 0;

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

    /// Record a match result and advance the winner through the bracket.
    ///
    /// Returns `Some(MatchStartAction)` if the next match is now ready to play,
    /// or `None` if waiting for more results or the tournament is complete.
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

            // Validate the winner is actually in this match
            if m.player1_id != Some(winner_id) && m.player2_id != Some(winner_id) {
                return Err("Winner is not a participant of this match".into());
            }

            if m.is_complete() {
                return Err("Match already completed".into());
            }

            m.winner_id = Some(winner_id);
            (m.round, m.position)
        };

        // Check if this was the final round
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

    /// Forfeit a player from their current match, advancing the opponent.
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

    /// Find the match ID associated with a game session ID.
    pub fn find_match_by_game_id(&self, game_id: &str) -> Option<Uuid> {
        self.matches
            .iter()
            .find(|m| m.game_id.as_deref() == Some(game_id))
            .map(|m| m.id)
    }

    /// Look up a match by its ID.
    pub fn find_match(&self, match_id: Uuid) -> Option<&Match> {
        self.matches.iter().find(|m| m.id == match_id)
    }

    /// Find the active (incomplete) match for a given player.
    pub fn find_active_match_for_player(&self, player_id: Uuid) -> Option<&Match> {
        self.matches.iter().find(|m| {
            !m.is_complete()
                && m.game_id.is_some()
                && (m.player1_id == Some(player_id) || m.player2_id == Some(player_id))
        })
    }

    /// Build a serializable view of the current tournament state.
    pub fn view(&self) -> TournamentView {
        let winner = if self.state == TournamentState::Finished {
            // Find the final match winner
            self.matches
                .iter()
                .filter(|m| m.round + 1 == self.total_rounds && m.is_complete())
                .find_map(|m| {
                    m.winner_id
                        .and_then(|wid| self.find_player(wid).cloned())
                })
        } else {
            None
        };

        // Compute current_round as the lowest round with an active (incomplete, started) match
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

    /// Mark a player as disconnected.
    pub fn disconnect_player(&mut self, player_id: Uuid) {
        if let Some(p) = self.players.iter_mut().find(|p| p.id == player_id) {
            p.connected = false;
        }
    }

    /// Mark a player as reconnected.
    pub fn reconnect_player(&mut self, player_id: Uuid) {
        if let Some(p) = self.players.iter_mut().find(|p| p.id == player_id) {
            p.connected = true;
        }
    }

    /// Returns `true` if this tournament has been finished for longer than the TTL.
    pub fn is_stale(&self) -> bool {
        self.finished_at
            .map(|t| t.elapsed() > TOURNAMENT_TTL)
            .unwrap_or(false)
    }

    pub fn state(&self) -> &TournamentState {
        &self.state
    }
}

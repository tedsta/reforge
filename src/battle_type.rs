#[derive(RustcEncodable, RustcDecodable)]
pub enum BattleType {
    // Player vs player free for all
    FreeForAll {
        num_players: u8,
    }, 
    Ai, // Player vs AI
}

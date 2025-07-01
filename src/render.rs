use crate::game::{
    strip::{Square, StripIndex},
    GameState, Player,
};

pub fn render(game: &GameState) -> String {
    pub enum RowItem {
        I(u8),
        X,
    }
    use RowItem::*;
    const SIDE_ROW: [RowItem; 8] = [I(3), I(2), I(1), I(0), X, X, I(13), I(12)];
    const MID_ROW: [RowItem; 8] = [I(4), I(5), I(6), I(7), I(8), I(9), I(10), I(11)];

    fn draw_player(player: Option<Player>, square: Square) -> &'static str {
        match (player, square) {
            (Some(Player::Opp), Square::Flower) => "(O)",
            (Some(Player::Opp), Square::Normal) => "[O]",
            (Some(Player::Prot), Square::Flower) => "(P)",
            (Some(Player::Prot), Square::Normal) => "[P]",
            (None, Square::Normal) => " - ",
            (None, Square::Flower) => " * ",
        }
    }
    let side_row_prot: String = SIDE_ROW
        .iter()
        .map(|item| match item {
            I(i) => {
                let index = StripIndex::new(*i).unwrap();
                if game.prot.strip.get(index) {
                    draw_player(Some(Player::Prot), index.square())
                } else {
                    draw_player(None, index.square())
                }
            }
            X => "   ",
        })
        .collect();
    let side_row_opp: String = SIDE_ROW
        .iter()
        .map(|item| match item {
            I(i) => {
                let index = StripIndex::new(*i).unwrap();
                if game.opp.strip.get(index) {
                    draw_player(Some(Player::Opp), index.square())
                } else {
                    draw_player(None, index.square())
                }
            }
            X => "   ",
        })
        .collect();
    let mid_row: String = MID_ROW
        .iter()
        .map(|item| match item {
            I(i) => {
                let index = StripIndex::new(*i).unwrap();
                draw_player(game.player_at_i(index), index.square())
            }
            X => "   ",
        })
        .collect();
    let prot_score = game.prot.score;
    let prot_turn = if game.first_player_is_prot {
        "has turn"
    } else {
        ""
    };
    let opp_score = game.opp.score;
    let opp_turn = if !game.first_player_is_prot {
        "has turn"
    } else {
        ""
    };

    format!("{side_row_prot}   score: {prot_score} {prot_turn} \n{mid_row}\n{side_row_opp}   score: {opp_score} {opp_turn}", )
}

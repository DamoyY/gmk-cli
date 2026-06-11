use gmk_cli::board::Board;
use gmk_cli::coordinate::{BOARD_CELLS, BOARD_SIZE, Coordinate, axis_label, parse_axis};
use gmk_cli::errors::IllegalMove;
use gmk_cli::persistence::{decode_board, encode_board};
use gmk_cli::stone::Stone;
#[test]
fn coordinate_labels_cover_the_standard_board() {
    assert_eq!(BOARD_SIZE, 15);
    assert_eq!(BOARD_CELLS, 225);
    assert_eq!(axis_label(0), 'a');
    assert_eq!(axis_label(14), 'o');
    assert_eq!(parse_axis("row", "a").unwrap(), 0);
    assert_eq!(parse_axis("row", "o").unwrap(), 14);
    assert_eq!(parse_axis("column", "1").unwrap(), 0);
    assert_eq!(parse_axis("column", "15").unwrap(), 14);
    assert_result_is_err(&parse_axis("row", "p"));
    assert_result_is_err(&parse_axis("row", "0"));
    assert_result_is_err(&parse_axis("row", "16"));
}
#[test]
fn first_move_is_black_and_legal_moves_alternate() {
    let mut board = Board::empty();
    let first = board.place(Coordinate::parse("a", "a").unwrap()).unwrap();
    let second = board.place(Coordinate::parse("a", "b").unwrap()).unwrap();
    let third = board.place(Coordinate::parse("b", "a").unwrap()).unwrap();
    assert_eq!(first.sequence, 1);
    assert_eq!(first.stone, Stone::Black);
    assert_eq!(second.sequence, 2);
    assert_eq!(second.stone, Stone::White);
    assert_eq!(third.sequence, 3);
    assert_eq!(third.stone, Stone::Black);
    assert_eq!(board.moves(), 3);
}
#[test]
fn occupied_positions_are_illegal_without_changing_turn() {
    let mut board = Board::empty();
    let coordinate = Coordinate::parse("c", "d").unwrap();
    board.place(coordinate).unwrap();
    let error = board.place(coordinate).unwrap_err();
    assert_eq!(error, IllegalMove::Occupied { coordinate });
    let next = board.place(Coordinate::parse("c", "e").unwrap()).unwrap();
    assert_eq!(next.stone, Stone::White);
}
#[test]
fn rendered_board_is_ascii_csv_with_full_headers() {
    let mut board = Board::empty();
    board.place(Coordinate::parse("a", "a").unwrap()).unwrap();
    board.place(Coordinate::parse("b", "c").unwrap()).unwrap();
    let rendered = board.render_csv();
    let lines = rendered.lines().collect::<Vec<_>>();
    assert_eq!(
        lines.first().copied(),
        Some("#, a, b, c, d, e, f, g, h, i, j, k, l, m, n, o"),
    );
    assert_eq!(lines.len(), 16);
    assert_eq!(
        lines.get(1).map(|line| line.starts_with("a, 0, *, *, *")),
        Some(true),
    );
    assert_eq!(
        lines.get(2).map(|line| line.starts_with("b, *, *, 1, *")),
        Some(true),
    );
    assert!(rendered.is_ascii());
}
#[test]
fn persisted_board_round_trips_and_rejects_corruption() {
    let mut board = Board::empty();
    board.place(Coordinate::parse("a", "a").unwrap()).unwrap();
    board.place(Coordinate::parse("o", "o").unwrap()).unwrap();
    let encoded = encode_board(&board);
    let decoded = decode_board(std::path::Path::new("state.txt"), &encoded).unwrap();
    assert_eq!(decoded, board);
    let corrupt = encoded.replace("moves 2", "moves 3");
    assert_result_is_err(&decode_board(std::path::Path::new("state.txt"), &corrupt));
}
fn assert_result_is_err<T, E>(result: &Result<T, E>) {
    let is_error = result.is_err();
    assert!(is_error, "expected an error result");
}

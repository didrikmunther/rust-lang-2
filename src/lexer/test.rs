use super::*;

#[test]
fn unexpected_end_of_string() {
    let lexer = Lexer::new();

    assert_matches!(
        lexer.strip_strings(Block::new(BlockType::Rest, String::from("Hello, \"there \"        \" ").chars().collect(), 0)),
        Err(Error { error_type: ErrorType::LexerError(LexerErrorType::UnexpectedEndOfString), pos: 23, width: 1, .. })
    );
}

#[test]
fn removes_strings() {
    let lexer = Lexer::new();

    let result = lexer.strip_strings(Block::new(BlockType::Rest, String::from("Hello, \"there \" handsome").chars().collect(), 0));
    assert!(result.is_ok());

    let mut unwrapped = result.unwrap().into_iter();

    assert_matches!(unwrapped.next().unwrap(), Block {
        block_type: BlockType::Rest,
        offset: 0,
        width: 7, ..
    });
    assert_matches!(unwrapped.next().unwrap(), Block {
        block_type: BlockType::Literal(Literal::String(_)),
        offset: 8,
        width: 6, ..
    });
    assert_matches!(unwrapped.next().unwrap(), Block {
        block_type: BlockType::Rest,
        offset: 15,
        width: 9, ..
    });
}

#[test]
fn comments_work() {
    let lexer = Lexer::new();

    let wo_strings = lexer.strip_strings(Block::new(BlockType::Rest, String::from("Hello, \"the//re \" handsome // this is a comment\n//2nd comment \"string 2\"").chars().collect(), 0));
    assert!(wo_strings.is_ok());
    let result = lexer.replace_rest(wo_strings.unwrap(), &Lexer::strip_comments);
    assert!(result.is_ok());

    let mut unwrapped = result.unwrap().into_iter();

    assert_matches!(unwrapped.next().unwrap(), Block {
        block_type: BlockType::Rest,
        offset: 0,
        width: 7, ..
    });

    assert_matches!(unwrapped.next().unwrap(), Block {
        block_type: BlockType::Literal(Literal::String(_)),
        offset: 8,
        width: 8, ..
    });

    assert_matches!(unwrapped.next().unwrap(), Block {
        block_type: BlockType::Rest,
        offset: 17,
        width: 10, ..
    });

    assert_matches!(unwrapped.next().unwrap(), Block {
        block_type: BlockType::Comment,
        offset: 29,
        width: 18, ..
    });

    assert_matches!(unwrapped.next().unwrap(), Block {
        block_type: BlockType::Comment,
        offset: 50,
        width: 22, ..
    });
}
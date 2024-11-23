use slickcmd::calculator;

#[test]
fn test_accepts_input() {
    let input = "a=2;a";
    let result = calculator::accepts_input(input);
    assert!(result);
}
#![cfg(feature = "code-obfuscation")]

use ferrumward_macros::obfuscate;

#[obfuscate]
fn compute_secret(a: i32, b: i32) -> i32 {
    let mut sum = 0;
    for i in 0..10 {
        sum += a + b + i;
    }
    sum
}

#[test]
fn test_obfuscation_macro() {
    let result = compute_secret(5, 10);
    // Expected: 10 * (5+10) + sum(0..9) = 150 + 45 = 195
    assert_eq!(result, 195);
}

//

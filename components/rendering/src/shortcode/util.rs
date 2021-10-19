/// Macro which handles reduces a lot of the messiness when expecting a certain token from a lexer.
macro_rules! expect_token {
    ($token:expr, [$($expected:pat => $response:expr),+$(,)?], $none_err:expr, $($err_variant_fn:expr)?$(,)?) => {
        match $token {
            $(Some($expected) => Ok($response)),+,
            $(Some(t) => Err($err_variant_fn(t)))?,
            None => Err($none_err),
        }
    }
}

#[cfg(test)]
#[macro_use]
mod lex_test_macros {
    macro_rules! lex_assert_eq_ok {
        ($token_ty:ty, $ty:ty, $content:expr, $eq:expr$(, $rem:expr)?) => {
            #[allow(unused_mut)]
            let mut lexer = <$token_ty>::lexer($content);
            #[allow(unused_mut)]
            let (mut lexer, val) = <$ty>::lex_parse(lexer).expect("Expected to be Ok");
            assert_eq!(val, $eq);
            $(assert_eq!(lexer.remainder(), $rem);)?
        }
    }

    macro_rules! lex_assert_eq_err {
        ($token_ty:ty, $ty:ty, $content:expr, $eq:expr) => {
            #[allow(unused_mut)]
            let mut lexer = <$token_ty>::lexer($content);
            assert_eq!(
                <$ty>::lex_parse(lexer).map(|_| ()).expect_err("Expected to be Err"),
                $eq
            );
        };
    }
}

use defmt::Format;
use error_set::error_set;

error_set! {
    #[derive(Format)]
    ParsingError = {
        Empty,
        InvalidCommandFormat,
        InvalidArgumentFormat,
        TooManyArguments
    };
    #[derive(Format)]
    DecodingError = {
        UnknownCommand,
        NotEnoughArguments,
        TooManyArguments,
        InvalidArgumentName,
        InvalidArgumentValue,
    };
    #[derive(Format)]
    CommandChainError = ParsingError || DecodingError;
}

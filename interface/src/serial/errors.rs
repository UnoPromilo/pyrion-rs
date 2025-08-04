use defmt::Format;
use error_set::error_set;
use heapless::String;

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
        NotEnoughArguments{
            missing_argument: String<20>,
        },
        InvalidArgumentValue,
    };
    #[derive(Format)]
    CommandChainError = ParsingError || DecodingError;
}

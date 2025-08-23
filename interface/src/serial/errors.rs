use error_set::error_set;

error_set! {
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    ParsingError = {
        Empty,
        InvalidCommandFormat,
        InvalidArgumentFormat,
        TooManyArguments
    };
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    DecodingError = {
        UnknownCommand,
        NotEnoughArguments,
        TooManyArguments,
        InvalidArgumentName,
        InvalidArgumentValue,
    };
    #[cfg_attr(feature = "defmt", derive(defmt::Format))]
    CommandChainError = ParsingError || DecodingError;
}

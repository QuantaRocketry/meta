use crate::info;
use at_commands::parser::CommandParser;

pub enum AtCommand {
    Ping,
    GetFrequency,
    SetFrequency(i32),
    SetSF(u8),
    GetSF,
}

pub fn handle_command(line: &[u8]) -> Option<AtCommand> {
    if CommandParser::parse(line)
        .expect_identifier(b"AT?")
        .finish()
        .is_ok()
    {
        info!("CLI: AT?");

        return Some(AtCommand::Ping);
    }

    if CommandParser::parse(line)
        .expect_identifier(b"AT+Frequency?")
        .finish()
        .is_ok()
    {
        info!("CLI: AT+Frequency?");

        return Some(AtCommand::GetFrequency);
    }

    if let Ok((hz,)) = CommandParser::parse(line)
        .expect_identifier(b"AT+Frequency=")
        .expect_int_parameter()
        .finish()
    {
        info!("CLI: AT+Frequency={}", &hz);

        // radio.set_freq_hz(hz as u32); // your radio driver call
        return Some(AtCommand::SetFrequency(hz));
    }

    if CommandParser::parse(line)
        .expect_identifier(b"AT+Frequency=?")
        .finish()
        .is_ok()
    {
        info!("CLI: AT+Frequency?");

        return None;
    }

    None
}

use anyhow::anyhow;
use std::error::Error;
use tango_controls_client_sys::device_proxy::DeviceProxy;
use tango_controls_client_sys::types::{CmdArgType, CommandData, CommandInfo, DevState};

use crate::tango_utils::_split_strip_string;

pub fn get_command_list(device_name: &str) -> Result<Vec<CommandInfo>, Box<dyn Error>> {
    let dp = DeviceProxy::new(device_name)?;
    let attributes = dp.command_list_query()?;
    Ok(attributes)
}

pub fn _get_command_details(
    device_proxy: &DeviceProxy,
    command_name: &str,
) -> Result<CommandInfo, Box<dyn Error>> {
    let command_info = device_proxy.command_query(command_name)?;
    Ok(command_info)
}

pub fn _execute_tango_command(
    device_name: &str,
    command_name: &str,
    paramater: &str,
) -> Result<CommandData, Box<dyn Error>> {
    let dp = DeviceProxy::new(device_name)?;
    let command_info = _get_command_details(&dp, command_name)?;
    let parsed_paramater = _parse_command_data(paramater, command_info.in_type)?;
    let res = dp.command_inout(command_name, parsed_paramater)?;
    Ok(res)
}

pub fn _parse_command_data(
    data: &str,
    data_type: CmdArgType,
) -> Result<CommandData, Box<dyn Error>> {
    let res = match data_type {
        CmdArgType::DevVoid => CommandData::Void,
        CmdArgType::DevBoolean => match data {
            "True" | "true" | "1" => CommandData::Boolean(true),
            _ => CommandData::Boolean(false),
        },
        CmdArgType::DevShort => {
            let short: i16 = data.parse()?;
            CommandData::Short(short)
        }
        CmdArgType::DevLong => {
            let long: i32 = data.parse()?;
            CommandData::Long(long)
        }
        CmdArgType::DevFloat => {
            let float: f32 = data.parse()?;
            CommandData::Float(float)
        }
        CmdArgType::DevDouble => {
            let double: f64 = data.parse()?;
            CommandData::Double(double)
        }
        CmdArgType::DevUShort => {
            let ushort: u16 = data.parse()?;
            CommandData::UShort(ushort)
        }
        CmdArgType::DevULong => {
            let ulong: u32 = data.parse()?;
            CommandData::ULong(ulong)
        }
        CmdArgType::DevString => CommandData::String(data.to_string()),
        CmdArgType::DevVarCharArray => {
            let s: Vec<char> = data.chars().filter(|&c| c.is_numeric()).collect();
            let ca: Vec<u8> = s.into_iter().map(|c| c as u8 - 48).collect();
            if ca.is_empty() {
                return Err(anyhow!("A value is required").into());
            }
            CommandData::CharArray(ca)
        }
        CmdArgType::DevVarShortArray => {
            let strip_c = _split_strip_string(data);
            let mut sa: Vec<i16> = Vec::new();
            for c in strip_c.iter() {
                let parsed_c: i16 = c.parse()?;
                sa.push(parsed_c);
            }
            if sa.is_empty() {
                return Err(anyhow!("A value is required").into());
            }
            CommandData::ShortArray(sa)
        }
        CmdArgType::DevVarLongArray => {
            let strip_c = _split_strip_string(data);
            let mut longa: Vec<i32> = Vec::new();
            for c in strip_c.iter() {
                let parsed_c: i32 = c.parse()?;
                longa.push(parsed_c);
            }
            if longa.is_empty() {
                return Err(anyhow!("A value is required").into());
            }
            CommandData::LongArray(longa)
        }
        CmdArgType::DevVarFloatArray => {
            let strip_c = _split_strip_string(data);
            let mut fa: Vec<f32> = Vec::new();
            for c in strip_c.iter() {
                let parsed_c: f32 = c.parse()?;
                fa.push(parsed_c);
            }
            if fa.is_empty() {
                return Err(anyhow!("A value is required").into());
            }
            CommandData::FloatArray(fa)
        }
        CmdArgType::DevVarDoubleArray => {
            let strip_c = _split_strip_string(data);
            let mut da: Vec<f64> = Vec::new();
            for c in strip_c.iter() {
                let parsed_c: f64 = c.parse()?;
                da.push(parsed_c);
            }
            if da.is_empty() {
                return Err(anyhow!("A value is required").into());
            }
            CommandData::DoubleArray(da)
        }
        CmdArgType::DevVarUShortArray => {
            let strip_c = _split_strip_string(data);
            let mut usa: Vec<u16> = Vec::new();
            for c in strip_c.iter() {
                let parsed_c: u16 = c.parse()?;
                usa.push(parsed_c);
            }
            if usa.is_empty() {
                return Err(anyhow!("A value is required").into());
            }
            CommandData::UShortArray(usa)
        }
        CmdArgType::DevVarULongArray => {
            let strip_c = _split_strip_string(data);
            let mut ula: Vec<u32> = Vec::new();
            for c in strip_c.iter() {
                let parsed_c: u32 = c.parse()?;
                ula.push(parsed_c);
            }
            if ula.is_empty() {
                return Err(anyhow!("A value is required").into());
            }
            CommandData::ULongArray(ula)
        }
        CmdArgType::DevState => {
            let state = match data {
                "ON" => Ok(DevState::ON),
                "OFF" => Ok(DevState::OFF),
                "CLOSE" => Ok(DevState::CLOSE),
                "OPEN" => Ok(DevState::OPEN),
                "INSERT" => Ok(DevState::INSERT),
                "EXTRACT" => Ok(DevState::EXTRACT),
                "MOVING" => Ok(DevState::MOVING),
                "STANDBY" => Ok(DevState::STANDBY),
                "FAULT" => Ok(DevState::FAULT),
                "INIT" => Ok(DevState::INIT),
                "RUNNING" => Ok(DevState::RUNNING),
                "ALARM" => Ok(DevState::ALARM),
                "DISABLE" => Ok(DevState::DISABLE),
                "UNKNOWN" => Ok(DevState::UNKNOWN),
                _ => return Err(anyhow!("State not recognised").into()),
            };
            match state {
                Ok(res) => CommandData::DevState(res),
                Err(err) => err,
            }
        }
        CmdArgType::DevVarBooleanArray => {
            let strip_c = _split_strip_string(data);
            let mut ba: Vec<bool> = Vec::new();
            for c in strip_c.iter().map(|s| s.as_str()) {
                match c {
                    "true" | "True" | "1" => ba.push(true),
                    _ => ba.push(false),
                }
            }
            if ba.is_empty() {
                return Err(anyhow!("A value is required").into());
            }
            CommandData::BooleanArray(ba)
        }
        CmdArgType::DevLong64 => {
            let long64: i64 = data.parse()?;
            CommandData::Long64(long64)
        }
        CmdArgType::DevULong64 => {
            let long: u64 = data.parse()?;
            CommandData::ULong64(long)
        }
        CmdArgType::DevVarLong64Array => {
            let strip_c = _split_strip_string(data);
            let mut la: Vec<i64> = Vec::new();
            for c in strip_c.iter() {
                let parsed_c: i64 = c.parse()?;
                la.push(parsed_c);
            }
            if la.is_empty() {
                return Err(anyhow!("A value is required").into());
            }
            CommandData::Long64Array(la)
        }
        CmdArgType::DevVarULong64Array => {
            let strip_c = _split_strip_string(data);
            let mut la: Vec<u64> = Vec::new();
            for c in strip_c.iter() {
                let parsed_c: u64 = c.parse()?;
                la.push(parsed_c);
            }
            if la.is_empty() {
                return Err(anyhow!("A value is required").into());
            }
            CommandData::ULong64Array(la)
        }
        _ => return Err(anyhow!("Command input type [{:?}] not supported", data_type).into()),
    };
    Ok(res)
}

#[cfg(test)]
mod command_tests {
    use super::*;

    #[test]
    fn test_command_param_parse() {
        use tango_controls_client_sys::types::{CmdArgType, CommandData};

        let tests = vec![
            (
                "Void",
                _parse_command_data("", CmdArgType::DevVoid),
                CommandData::Void,
            ),
            (
                "Boolean",
                _parse_command_data("true", CmdArgType::DevBoolean),
                CommandData::Boolean(true),
            ),
            (
                "Short",
                _parse_command_data("-147", CmdArgType::DevShort),
                CommandData::Short(-147),
            ),
            (
                "Long",
                _parse_command_data("-1048576", CmdArgType::DevLong),
                CommandData::Long(-(1 << 20)),
            ),
            (
                "Float",
                _parse_command_data("42.42", CmdArgType::DevFloat),
                CommandData::Float(42.42),
            ),
            (
                "Double",
                _parse_command_data("123.456790123752", CmdArgType::DevDouble),
                CommandData::Double(123.456790123752),
            ),
            (
                "UShort",
                _parse_command_data("137", CmdArgType::DevUShort),
                CommandData::UShort(137),
            ),
            (
                "ULong",
                _parse_command_data("1048576", CmdArgType::DevULong),
                CommandData::ULong(1 << 20),
            ),
            (
                "Long64",
                _parse_command_data("-1152921504606846976", CmdArgType::DevLong64),
                CommandData::Long64(-(1 << 60)),
            ),
            (
                "ULong64",
                _parse_command_data("1152921504606846976", CmdArgType::DevULong64),
                CommandData::ULong64(1 << 60),
            ),
            (
                "String",
                _parse_command_data("some_str_ing", CmdArgType::DevString),
                CommandData::String("some_str_ing".to_string()),
            ),
            (
                "CharArray",
                _parse_command_data("1 5 7", CmdArgType::DevVarCharArray),
                CommandData::CharArray(vec![1, 5, 7]),
            ),
            (
                "CharArrayComma",
                _parse_command_data("1, 5, 7", CmdArgType::DevVarCharArray),
                CommandData::CharArray(vec![1, 5, 7]),
            ),
            (
                "CharArrayBrackets",
                _parse_command_data("[1, 5, 7]", CmdArgType::DevVarCharArray),
                CommandData::CharArray(vec![1, 5, 7]),
            ),
            (
                "CharArraySpaces",
                _parse_command_data("  [  1  ,   5  ,   7  ]  ", CmdArgType::DevVarCharArray),
                CommandData::CharArray(vec![1, 5, 7]),
            ),
            (
                "ShortArray",
                _parse_command_data("-5, 1, 0", CmdArgType::DevVarShortArray),
                CommandData::ShortArray(vec![-5, 1, 0]),
            ),
            (
                "UShortArray",
                _parse_command_data("5, 1, 0", CmdArgType::DevVarUShortArray),
                CommandData::UShortArray(vec![5, 1, 0]),
            ),
            (
                "LongArray",
                _parse_command_data("-1048576, 1, 0", CmdArgType::DevVarLongArray),
                CommandData::LongArray(vec![-(1 << 20), 1, 0]),
            ),
            (
                "ULongArray",
                _parse_command_data("1073741824, 1, 0", CmdArgType::DevVarULongArray),
                CommandData::ULongArray(vec![1 << 30, 1, 0]),
            ),
            (
                "Long64Array",
                _parse_command_data("-1152921504606846976, 1, 0", CmdArgType::DevVarLong64Array),
                CommandData::Long64Array(vec![-(1 << 60), 1, 0]),
            ),
            (
                "ULong64Array",
                _parse_command_data("1152921504606846976, 1, 0", CmdArgType::DevVarULong64Array),
                CommandData::ULong64Array(vec![1 << 60, 1, 0]),
            ),
            (
                "FloatArray",
                _parse_command_data("-42.4, 0.0, 80.123", CmdArgType::DevVarFloatArray),
                CommandData::FloatArray(vec![-42.4, 0.0, 80.123]),
            ),
            (
                "DoubleArray",
                _parse_command_data("-5.0, 1.0, 0.0", CmdArgType::DevVarDoubleArray),
                CommandData::DoubleArray(vec![-5.0, 1.0, 0.0]),
            ),
        ];
        for (dtype, res, data) in tests {
            println!("{}", dtype);
            assert_eq!(res.unwrap(), data);
        }
    }

    #[test]
    fn test_command_executions() {
        let dev = DeviceProxy::new("sys/tg_test/1")
            .expect("Could not proxy to sys/tg_test/1, is a database running on localhost?");

        // test all types
        println!("\nTesting commands for all data types:");
        let tests = vec![
            ("DevVoid", _parse_command_data("", CmdArgType::DevVoid)),
            (
                "DevBoolean",
                _parse_command_data("true", CmdArgType::DevBoolean),
            ),
            (
                "DevShort",
                _parse_command_data("-147", CmdArgType::DevShort),
            ),
            (
                "DevLong",
                _parse_command_data("-1048576", CmdArgType::DevLong),
            ),
            (
                "DevFloat",
                _parse_command_data("42.42", CmdArgType::DevFloat),
            ),
            (
                "DevDouble",
                _parse_command_data("123.456790123752", CmdArgType::DevDouble),
            ),
            (
                "DevUShort",
                _parse_command_data("137", CmdArgType::DevUShort),
            ),
            (
                "DevULong",
                _parse_command_data("1048576", CmdArgType::DevULong),
            ),
            (
                "DevLong64",
                _parse_command_data("-1152921504606846976", CmdArgType::DevLong64),
            ),
            (
                "DevULong64",
                _parse_command_data("1152921504606846976", CmdArgType::DevULong64),
            ),
            (
                "DevString",
                _parse_command_data("some_str_ing", CmdArgType::DevString),
            ),
            ("DevString", _parse_command_data("", CmdArgType::DevString)),
            (
                "DevVarCharArray",
                _parse_command_data("1 5 7", CmdArgType::DevVarCharArray),
            ),
            (
                "DevVarShortArray",
                _parse_command_data("-5, 1, 0", CmdArgType::DevVarShortArray),
            ),
            (
                "DevVarUShortArray",
                _parse_command_data("5, 1, 0", CmdArgType::DevVarUShortArray),
            ),
            (
                "DevVarLongArray",
                _parse_command_data("-1048576, 1, 0", CmdArgType::DevVarLongArray),
            ),
            (
                "DevVarULongArray",
                _parse_command_data("1073741824, 1, 0", CmdArgType::DevVarULongArray),
            ),
            (
                "DevVarLong64Array",
                _parse_command_data("-1152921504606846976, 1, 0", CmdArgType::DevVarLong64Array),
            ),
            (
                "DevVarULong64Array",
                _parse_command_data("1152921504606846976, 1, 0", CmdArgType::DevVarULong64Array),
            ),
            (
                "DevVarFloatArray",
                _parse_command_data("-42.4, 0.0, 80.123", CmdArgType::DevVarFloatArray),
            ),
            (
                "DevVarDoubleArray",
                _parse_command_data("-5.0, 1.0, 0.0", CmdArgType::DevVarDoubleArray),
            ),
        ];
        for (cmd, data) in tests {
            println!("{}", cmd);
            let data = data.unwrap();
            let res = dev.command_inout(cmd, data.clone()).expect(
                "Could not execute command on sys/tg_test/1, is \
                                  the TangoTest server running?",
            );
            assert_eq!(res, data);
        }
    }

    #[test]
    fn test_commands() -> Result<(), Box<dyn Error>> {
        let test_strings = vec![
            ("DevVoid", ""),
            ("DevBoolean", "true"),
            ("DevShort", "-147"),
            ("DevLong", "-1048576"),
            ("DevFloat", "42.42"),
            ("DevDouble", "123.456790123752"),
            ("DevUShort", "137"),
            ("DevULong", "1048576"),
            ("DevLong64", "-1152921504606846976"),
            ("DevULong64", "1152921504606846976"),
            ("DevString", "some_str_ing"),
            ("DevVarCharArray", "[1, 5, 7]"),
            ("DevVarShortArray", "[-5, 1, 0]"),
            ("DevVarUShortArray", "[5, 1, 0]"),
            ("DevVarLongArray", "[-1048576, 1, 0]"),
            ("DevVarULongArray", "[1073741824, 1, 0]"),
            ("DevVarLong64Array", "[-1152921504606846976, 1, 0]"),
            ("DevVarULong64Array", "[1152921504606846976, 1, 0]"),
            ("DevVarFloatArray", "[-42.4, 0, 80.123]"),
            ("DevVarDoubleArray", "[-5, 1, 0]"),
        ];

        for (cmd, data) in test_strings {
            println!("Command: {}, Value: {}", cmd, data);
            let command_data_res = _execute_tango_command("sys/tg_test/1", cmd, data);
            match command_data_res {
                Ok(command_data) => println!("{:?}", command_data),
                Err(err) => {
                    println!("{}", err);
                    return Err(err);
                }
            }
        }

        let not_supported = vec![
            ("DevVarStringArray", "[ab, c, d]", "DevVarStringArray"),
            (
                "DevVarLongStringArray",
                "[-5, 1, 0, 1][ab, c]",
                "DevVarLongStringArray",
            ),
            (
                "DevVarDoubleStringArray",
                "[-5, 1, 0][ab, c]",
                "DevVarDoubleStringArray",
            ),
        ];

        for (cmd, data, type_str) in not_supported {
            println!("Command: {}, Value: {}", cmd, data);
            let command_data_res = _execute_tango_command("sys/tg_test/1", cmd, data);
            let error = command_data_res.unwrap_err();
            assert_eq!(
                error.to_string(),
                format!("Command input type [{}] not supported", type_str)
            );
        }
        Ok(())
    }
}

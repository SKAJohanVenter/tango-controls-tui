use anyhow::anyhow;
use log::error;
use ratatui_tree_widget::TreeItem;
use std::{collections::BTreeMap, error::Error};
use tango_controls_client_sys::database_proxy::DatabaseProxy;
use tango_controls_client_sys::device_proxy::DeviceProxy;
use tango_controls_client_sys::types::{
    AttrDataFormat, AttrValue, AttributeData, AttributeInfo, CmdArgType, CommandData, CommandInfo,
    DevState,
};

pub struct DeviceAttribute {
    pub attribute_info: AttributeInfo,
    pub attribute_data: Option<AttributeData>,
}

#[derive(Debug, Default, Clone)]
pub struct Member {
    pub device_name: String,
}

#[derive(Debug, Default, Clone)]
pub struct Family {
    pub members: BTreeMap<String, Member>,
}

#[derive(Debug, Default, Clone)]
pub struct Domain {
    pub families: BTreeMap<String, Family>,
}

#[derive(Debug, Default, Clone)]
pub struct TangoDevicesLookup<'a> {
    pub domains: BTreeMap<String, Domain>,
    pub devices: Vec<String>,
    pub tree_items: Vec<TreeItem<'a>>,
}

pub trait GetTreeItems<'a> {
    fn get_tree_items(&self) -> Vec<TreeItem<'a>>;
}

impl<'a> GetTreeItems<'a> for Member {
    fn get_tree_items(&self) -> Vec<TreeItem<'a>> {
        vec![TreeItem::new_leaf(self.device_name.clone())]
    }
}

impl<'a> GetTreeItems<'a> for Family {
    fn get_tree_items(&self) -> Vec<TreeItem<'a>> {
        let items = self
            .members
            .values()
            .map(|member| TreeItem::new_leaf(member.device_name.clone()))
            .collect::<Vec<TreeItem<'a>>>();
        items
    }
}

impl<'a> GetTreeItems<'a> for Domain {
    fn get_tree_items(&self) -> Vec<TreeItem<'a>> {
        let mut items: Vec<TreeItem<'a>> = Vec::new();
        for (family_key, family) in &self.families {
            items.push(TreeItem::new(family_key.clone(), family.get_tree_items()))
        }
        items
    }
}

impl<'a> GetTreeItems<'a> for TangoDevicesLookup<'a> {
    fn get_tree_items(&self) -> Vec<TreeItem<'a>> {
        let mut items: Vec<TreeItem<'a>> = Vec::new();
        for (domain_key, domain) in &self.domains {
            items.push(TreeItem::new(domain_key.clone(), domain.get_tree_items()))
        }
        items
    }
}

impl Family {
    pub fn get_by_ix(&self, ix: usize) -> Option<Member> {
        let member_keys: Vec<String> = self.members.keys().cloned().collect();
        if let Some(member_key) = member_keys.get(ix) {
            if let Some(member) = self.members.get(member_key) {
                return Some(member.clone());
            }
        }
        None
    }
}

impl Domain {
    pub fn get_by_ix(&self, ix: usize) -> Option<Family> {
        let family_keys: Vec<String> = self.families.keys().cloned().collect();
        if let Some(family_key) = family_keys.get(ix) {
            if let Some(family) = self.families.get(family_key) {
                return Some(family.clone());
            }
        }
        None
    }
}

impl<'a> TangoDevicesLookup<'a> {
    pub fn get_by_ix(&self, ix: usize) -> Option<Domain> {
        let domain_keys: Vec<String> = self.domains.keys().cloned().collect();
        if let Some(domain_key) = domain_keys.get(ix) {
            if let Some(domain) = self.domains.get(domain_key) {
                return Some(domain.clone());
            }
        }
        None
    }

    pub fn build() -> Result<TangoDevicesLookup<'a>, Box<dyn Error>> {
        let mut tdl = TangoDevicesLookup::default();
        let devices = TangoDevicesLookup::get_all_tango_devices()?;
        let domains = tdl.build_map(&devices);
        tdl.domains = domains;
        tdl.tree_items = tdl.get_tree_items();
        Ok(tdl)
    }

    pub fn get_all_tango_devices() -> Result<Vec<String>, Box<dyn Error>> {
        let dbp = DatabaseProxy::new()?;
        let dbdatum = dbp.get_device_exported("*")?;
        Ok(dbdatum)
    }

    pub fn split_devices_list<S: AsRef<str>>(devices_string: S) -> Vec<String> {
        let mut devices_str = devices_string.as_ref();
        if devices_str.is_empty() || devices_str == "[]" {
            vec![]
        } else {
            devices_str = &devices_str[1..];
            devices_str = &devices_str[..devices_str.len() - 1];
            let res = devices_str
                .split(", ")
                .map(|i| i.to_string())
                .collect::<Vec<String>>();
            res
        }
    }

    pub fn build_map(&mut self, devices: &[String]) -> BTreeMap<String, Domain> {
        let mut domains = BTreeMap::default();

        for device in devices {
            let split_device: Vec<&str> = device.split('/').collect();
            if let [domain_key, family_key, member_key] = split_device[..] {
                // Init the domains
                domains
                    .entry(domain_key.to_string())
                    .or_insert_with(Domain::default);

                if let Some(domain) = domains.get_mut(domain_key) {
                    // Init the families
                    domain
                        .families
                        .entry(family_key.to_string())
                        .or_insert_with(Family::default);

                    if let Some(family) = domain.families.get_mut(family_key) {
                        // Init the members
                        family
                            .members
                            .entry(member_key.to_string())
                            .or_insert(Member {
                                device_name: device.to_string(),
                            });
                    }
                }
            }
        }
        domains
    }
}

pub fn read_attribute(
    device_name: &str,
    attribute_name: &str,
) -> Result<Option<AttributeData>, Box<dyn Error>> {
    let dp = DeviceProxy::new(device_name)?;
    let attribute_data = match dp.read_attribute(attribute_name) {
        Ok(ad) => Some(ad),
        Err(err) => {
            error!(
                "Reading conversion error for {}/{}: {}",
                device_name, attribute_name, err
            );
            None
        }
    };
    Ok(attribute_data)
}

pub fn get_attribute_list(device_name: &str) -> Result<Vec<DeviceAttribute>, Box<dyn Error>> {
    let dp = DeviceProxy::new(device_name)?;
    let attributes = dp.attribute_list_query()?;
    let mut device_attributes: Vec<DeviceAttribute> = Vec::new();

    for attr in attributes {
        let attribute_data = match dp.read_attribute(&attr.name) {
            Ok(ad) => Some(ad),
            Err(err) => {
                error!(
                    "Reading conversion error for {}/{}: {}",
                    device_name, attr.name, err
                );
                None
            }
        };

        let da: DeviceAttribute = DeviceAttribute {
            attribute_data,
            attribute_info: attr,
        };
        device_attributes.push(da);
    }
    Ok(device_attributes)
}

pub fn get_command_list(device_name: &str) -> Result<Vec<CommandInfo>, Box<dyn Error>> {
    let dp = DeviceProxy::new(device_name)?;
    let attributes = dp.command_list_query()?;
    Ok(attributes)
}

pub fn get_command_details(
    device_proxy: &DeviceProxy,
    command_name: &str,
) -> Result<CommandInfo, Box<dyn Error>> {
    let command_info = device_proxy.command_query(command_name)?;
    Ok(command_info)
}

pub fn execute_tango_command(
    device_name: &str,
    command_name: &str,
    paramater: &str,
) -> Result<CommandData, Box<dyn Error>> {
    let dp = DeviceProxy::new(device_name)?;
    let command_info = get_command_details(&dp, command_name)?;
    let parsed_paramater = parse_command_data(paramater, command_info.in_type)?;
    let res = dp.command_inout(command_name, parsed_paramater)?;
    Ok(res)
}

pub fn split_strip_string(data: &str) -> Vec<String> {
    // Split on whitespace
    let cleaned_string: String = data
        .trim()
        .trim_matches('[')
        .trim_matches(']')
        .replace(',', ",  ");
    let split_w: Vec<&str> = cleaned_string.split_whitespace().collect();
    // Remove comma
    let mut strip_c: Vec<String> = split_w
        .iter()
        .map(|&s| s.replace(',', ""))
        .collect::<Vec<_>>();
    strip_c.retain(|s| !s.is_empty());
    strip_c
}

pub fn parse_command_data(
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
            let s: Vec<char> = data.chars().into_iter().filter(|&c| c.is_numeric()).collect();
            let ca: Vec<u8> = s.into_iter().map(|c| c as u8 - 48).collect();
            if ca.is_empty() {
                return Err(anyhow!("A value is required").into());
            }
            CommandData::CharArray(ca)
        }
        CmdArgType::DevVarShortArray => {
            let strip_c = split_strip_string(data);
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
            let strip_c = split_strip_string(data);
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
            let strip_c = split_strip_string(data);
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
            let strip_c = split_strip_string(data);
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
            let strip_c = split_strip_string(data);
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
            let strip_c = split_strip_string(data);
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
            let strip_c = split_strip_string(data);
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
            let strip_c = split_strip_string(data);
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
            let strip_c = split_strip_string(data);
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

pub fn display_attribute_type(attr_data_option: Option<AttributeData>) -> String {
    match attr_data_option {
        None => "N/A".to_string(),
        Some(attr_data) => match attr_data.data {
            AttrValue::Boolean(_) => "Boolean".to_string(),
            AttrValue::UChar(_) => "UChar".to_string(),
            AttrValue::Short(_) => "Short".to_string(),
            AttrValue::UShort(_) => "UShort".to_string(),
            AttrValue::Long(_) => "Long".to_string(),
            AttrValue::ULong(_) => "ULong".to_string(),
            AttrValue::Long64(_) => "Long64".to_string(),
            AttrValue::ULong64(_) => "ULong64".to_string(),
            AttrValue::Float(_) => "Float".to_string(),
            AttrValue::Double(_) => "Double".to_string(),
            AttrValue::String(_) => "String".to_string(),
            AttrValue::DevState(_) => "State".to_string(),
            AttrValue::DevEncoded(_) => "Encoded".to_string(),
            AttrValue::BooleanArray(_) => "BooleanArray".to_string(),
            AttrValue::UCharArray(_) => "UCharArray".to_string(),
            AttrValue::ShortArray(_) => "ShortArray".to_string(),
            AttrValue::UShortArray(_) => "UShortArray".to_string(),
            AttrValue::LongArray(_) => "LongArray".to_string(),
            AttrValue::ULongArray(_) => "ULongArray".to_string(),
            AttrValue::Long64Array(_) => "Long64Array".to_string(),
            AttrValue::ULong64Array(_) => "ULong64Array".to_string(),
            AttrValue::FloatArray(_) => "FloatArray".to_string(),
            AttrValue::DoubleArray(_) => "DoubleArray".to_string(),
            AttrValue::StringArray(_) => "StringArray".to_string(),
            AttrValue::DevStateArray(_) => "StateArray".to_string(),
            AttrValue::DevEncodedArray(_) => "EncodedArray".to_string(),
            AttrValue::DevEnum(_) => "DevEnum".to_string(),
            AttrValue::DevEnumArray(_) => "DevEnumArray".to_string(),
        },
    }
}

pub fn display_attribute_format(attr_type: AttrDataFormat) -> String {
    match attr_type {
        AttrDataFormat::SCALAR => "Scalar".to_string(),
        AttrDataFormat::SPECTRUM => "Spectrum".to_string(),
        AttrDataFormat::IMAGE => "Image".to_string(),
        _ => "Unknown".to_string(),
    }
}

#[cfg(test)]
mod tango_tests {
    use super::*;

    #[test]
    fn test_split_devices_list() {
        let empty: Vec<String> = Vec::new();
        let split_devices = TangoDevicesLookup::split_devices_list(String::from(""));
        assert_eq!(split_devices, empty);

        let split_devices = TangoDevicesLookup::split_devices_list(String::from("[]"));
        assert_eq!(split_devices, empty);

        let test_string = String::from("[a/b/c]");
        let split_devices = TangoDevicesLookup::split_devices_list(test_string);
        assert_eq!(split_devices, vec!["a/b/c"]);

        let test_string = String::from("[a/b/c, a/b/d, a/d/c, a/d/e, f/g/h]");
        let split_devices = TangoDevicesLookup::split_devices_list(test_string);
        assert_eq!(
            split_devices,
            vec![
                String::from("a/b/c"),
                String::from("a/b/d"),
                String::from("a/d/c"),
                String::from("a/d/e"),
                String::from("f/g/h"),
            ]
        )
    }

    #[test]
    fn test_split_strip_string() {
        let test_cases = vec![
            "1,2,3",
            " 1,2,3 ",
            "[1,2,3]",
            "1, 2, 3",
            "[1, 2, 3]",
            "[ 1, 2, 3 ]",
            " [ 1, 2, 3 ] ",
            " [ 1 , 2 ,  3 ] ",
        ];
        let expected_result = vec!["1".to_string(), "2".to_string(), "3".to_string()];
        for test_case in test_cases {
            assert_eq!(split_strip_string(test_case), expected_result)
        }
    }

    #[test]
    fn test_map_build() {
        let test_string = String::from("[a/b/c, a/b/d, a/d/c, a/d/e, f/g/h]");
        let split_devices = TangoDevicesLookup::split_devices_list(test_string);
        let mut map = TangoDevicesLookup::default();
        let domains = map.build_map(&split_devices);
        assert_eq!(
            domains
                .get("a")
                .unwrap()
                .families
                .get("d")
                .unwrap()
                .members
                .get("c")
                .unwrap()
                .device_name,
            "a/d/c"
        );
    }

    #[test]
    fn test_command_param_parse() {
        use tango_controls_client_sys::types::{CmdArgType, CommandData};

        let tests = vec![
            (
                "Void",
                parse_command_data("", CmdArgType::DevVoid),
                CommandData::Void,
            ),
            (
                "Boolean",
                parse_command_data("true", CmdArgType::DevBoolean),
                CommandData::Boolean(true),
            ),
            (
                "Short",
                parse_command_data("-147", CmdArgType::DevShort),
                CommandData::Short(-147),
            ),
            (
                "Long",
                parse_command_data("-1048576", CmdArgType::DevLong),
                CommandData::Long(-(1 << 20)),
            ),
            (
                "Float",
                parse_command_data("42.42", CmdArgType::DevFloat),
                CommandData::Float(42.42),
            ),
            (
                "Double",
                parse_command_data("123.456790123752", CmdArgType::DevDouble),
                CommandData::Double(123.456790123752),
            ),
            (
                "UShort",
                parse_command_data("137", CmdArgType::DevUShort),
                CommandData::UShort(137),
            ),
            (
                "ULong",
                parse_command_data("1048576", CmdArgType::DevULong),
                CommandData::ULong(1 << 20),
            ),
            (
                "Long64",
                parse_command_data("-1152921504606846976", CmdArgType::DevLong64),
                CommandData::Long64(-(1 << 60)),
            ),
            (
                "ULong64",
                parse_command_data("1152921504606846976", CmdArgType::DevULong64),
                CommandData::ULong64(1 << 60),
            ),
            (
                "String",
                parse_command_data("some_str_ing", CmdArgType::DevString),
                CommandData::String("some_str_ing".to_string()),
            ),
            (
                "CharArray",
                parse_command_data("1 5 7", CmdArgType::DevVarCharArray),
                CommandData::CharArray(vec![1, 5, 7]),
            ),
            (
                "CharArrayComma",
                parse_command_data("1, 5, 7", CmdArgType::DevVarCharArray),
                CommandData::CharArray(vec![1, 5, 7]),
            ),
            (
                "CharArrayBrackets",
                parse_command_data("[1, 5, 7]", CmdArgType::DevVarCharArray),
                CommandData::CharArray(vec![1, 5, 7]),
            ),
            (
                "CharArraySpaces",
                parse_command_data("  [  1  ,   5  ,   7  ]  ", CmdArgType::DevVarCharArray),
                CommandData::CharArray(vec![1, 5, 7]),
            ),
            (
                "ShortArray",
                parse_command_data("-5, 1, 0", CmdArgType::DevVarShortArray),
                CommandData::ShortArray(vec![-5, 1, 0]),
            ),
            (
                "UShortArray",
                parse_command_data("5, 1, 0", CmdArgType::DevVarUShortArray),
                CommandData::UShortArray(vec![5, 1, 0]),
            ),
            (
                "LongArray",
                parse_command_data("-1048576, 1, 0", CmdArgType::DevVarLongArray),
                CommandData::LongArray(vec![-(1 << 20), 1, 0]),
            ),
            (
                "ULongArray",
                parse_command_data("1073741824, 1, 0", CmdArgType::DevVarULongArray),
                CommandData::ULongArray(vec![1 << 30, 1, 0]),
            ),
            (
                "Long64Array",
                parse_command_data("-1152921504606846976, 1, 0", CmdArgType::DevVarLong64Array),
                CommandData::Long64Array(vec![-(1 << 60), 1, 0]),
            ),
            (
                "ULong64Array",
                parse_command_data("1152921504606846976, 1, 0", CmdArgType::DevVarULong64Array),
                CommandData::ULong64Array(vec![1 << 60, 1, 0]),
            ),
            (
                "FloatArray",
                parse_command_data("-42.4, 0.0, 80.123", CmdArgType::DevVarFloatArray),
                CommandData::FloatArray(vec![-42.4, 0.0, 80.123]),
            ),
            (
                "DoubleArray",
                parse_command_data("-5.0, 1.0, 0.0", CmdArgType::DevVarDoubleArray),
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
        let mut dev = DeviceProxy::new("sys/tg_test/1")
            .expect("Could not proxy to sys/tg_test/1, is a database running on localhost?");

        // test all types
        println!("\nTesting commands for all data types:");
        let tests = vec![
            ("DevVoid", parse_command_data("", CmdArgType::DevVoid)),
            (
                "DevBoolean",
                parse_command_data("true", CmdArgType::DevBoolean),
            ),
            ("DevShort", parse_command_data("-147", CmdArgType::DevShort)),
            (
                "DevLong",
                parse_command_data("-1048576", CmdArgType::DevLong),
            ),
            (
                "DevFloat",
                parse_command_data("42.42", CmdArgType::DevFloat),
            ),
            (
                "DevDouble",
                parse_command_data("123.456790123752", CmdArgType::DevDouble),
            ),
            (
                "DevUShort",
                parse_command_data("137", CmdArgType::DevUShort),
            ),
            (
                "DevULong",
                parse_command_data("1048576", CmdArgType::DevULong),
            ),
            (
                "DevLong64",
                parse_command_data("-1152921504606846976", CmdArgType::DevLong64),
            ),
            (
                "DevULong64",
                parse_command_data("1152921504606846976", CmdArgType::DevULong64),
            ),
            (
                "DevString",
                parse_command_data("some_str_ing", CmdArgType::DevString),
            ),
            ("DevString", parse_command_data("", CmdArgType::DevString)),
            (
                "DevVarCharArray",
                parse_command_data("1 5 7", CmdArgType::DevVarCharArray),
            ),
            (
                "DevVarShortArray",
                parse_command_data("-5, 1, 0", CmdArgType::DevVarShortArray),
            ),
            (
                "DevVarUShortArray",
                parse_command_data("5, 1, 0", CmdArgType::DevVarUShortArray),
            ),
            (
                "DevVarLongArray",
                parse_command_data("-1048576, 1, 0", CmdArgType::DevVarLongArray),
            ),
            (
                "DevVarULongArray",
                parse_command_data("1073741824, 1, 0", CmdArgType::DevVarULongArray),
            ),
            (
                "DevVarLong64Array",
                parse_command_data("-1152921504606846976, 1, 0", CmdArgType::DevVarLong64Array),
            ),
            (
                "DevVarULong64Array",
                parse_command_data("1152921504606846976, 1, 0", CmdArgType::DevVarULong64Array),
            ),
            (
                "DevVarFloatArray",
                parse_command_data("-42.4, 0.0, 80.123", CmdArgType::DevVarFloatArray),
            ),
            (
                "DevVarDoubleArray",
                parse_command_data("-5.0, 1.0, 0.0", CmdArgType::DevVarDoubleArray),
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
            let command_data_res = execute_tango_command("sys/tg_test/1", cmd, data);
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
            let command_data_res = execute_tango_command("sys/tg_test/1", cmd, data);
            let error = command_data_res.unwrap_err();
            assert_eq!(
                error.to_string(),
                format!("Command input type [{}] not supported", type_str)
            );
        }
        Ok(())
    }
}

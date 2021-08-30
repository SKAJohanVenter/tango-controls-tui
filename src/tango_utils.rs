use log::error;
use std::{collections::BTreeMap, error::Error};
use tango_client::*;
use tui_tree_widget::TreeItem;

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
        return vec![TreeItem::new_leaf(self.device_name.clone())];
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
        let devices_string = dbdatum.data.to_string();
        Ok(Self::split_devices_list(&devices_string))
    }

    pub fn split_devices_list<S: AsRef<str>>(devices_string: S) -> Vec<String> {
        let mut devices_str = devices_string.as_ref();
        if devices_str.len() == 0 || devices_str == "[]" {
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

        for device in devices.iter().cloned() {
            let split_device: Vec<&str> = device.split("/").collect();
            if let [domain_key, family_key, member_key] = split_device[..] {
                // Init the domains
                domains
                    .entry(domain_key.to_string())
                    .or_insert(Domain::default());

                if let Some(domain) = domains.get_mut(domain_key) {
                    // Init the families
                    domain
                        .families
                        .entry(family_key.to_string())
                        .or_insert(Family::default());

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
    let mut dp = DeviceProxy::new(device_name)?;
    // let value = dp.read_attribute(attribute_name)?;
    let attribute_data = match dp.read_attribute(&attribute_name) {
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

pub fn execute_command(
    device_name: &str,
    command: &str,
    parameter: &str,
) -> Result<CommandData, Box<dyn Error>> {
    let mut dp = DeviceProxy::new(device_name)?;
    let res = dp.command_inout(command, CommandData::from_str(parameter))?;
    Ok(res)
}

pub fn get_attribute_list(device_name: &str) -> Result<Vec<DeviceAttribute>, Box<dyn Error>> {
    let mut dp = DeviceProxy::new(device_name)?;
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
            attribute_data: attribute_data,
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
            AttrValue::State(_) => "State".to_string(),
            AttrValue::Encoded(_) => "Encoded".to_string(),
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
            AttrValue::StateArray(_) => "StateArray".to_string(),
            AttrValue::EncodedArray(_) => "EncodedArray".to_string(),
        },
    }
}

pub fn display_attribute_format(attr_type: AttrDataFormat) -> String {
    match attr_type {
        AttrDataFormat::Scalar => "Scalar".to_string(),
        AttrDataFormat::Spectrum => "Spectrum".to_string(),
        AttrDataFormat::Image => "Image".to_string(),
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commands() -> Result<(), Box<dyn Error>> {
        use tango_client::CommandData::*;

        let tango_host = env::var("TANGO_HOST")?;

        let mut dev = DeviceProxy::new(format!("tango://{}/sys/tg_test/1", tango_host).as_str())
            .expect("Could not proxy to sys/tg_test/1, is a database running on localhost?");

        // let tests = vec![
        //     ("DevVoid", Void),
        //     ("DevBoolean", Boolean(true)),
        //     ("DevShort", Short(-147)),
        //     ("DevLong", Long(-(1 << 20))),
        //     ("DevFloat", Float(42.42)),
        //     ("DevDouble", Double(123.456790123752)),
        //     ("DevUShort", UShort(137)),
        //     ("DevULong", ULong(1 << 20)),
        //     ("DevLong64", Long64(-(1 << 60))),
        //     ("DevULong64", ULong64(1 << 60)),
        //     ("DevString", CommandData::from_str("some_str_ing")),
        //     ("DevVarCharArray", CharArray(vec![1, 5, 7])),
        //     ("DevVarShortArray", ShortArray(vec![-5, 1, 0])),
        //     ("DevVarUShortArray", UShortArray(vec![5, 1, 0])),
        //     ("DevVarLongArray", LongArray(vec![-(1 << 20), 1, 0])),
        //     ("DevVarULongArray", ULongArray(vec![1 << 30, 1, 0])),
        //     ("DevVarLong64Array", Long64Array(vec![-(1 << 60), 1, 0])),
        //     ("DevVarULong64Array", ULong64Array(vec![1 << 60, 1, 0])),
        //     ("DevVarFloatArray", FloatArray(vec![-42.4, 0.0, 80.123])),
        //     ("DevVarDoubleArray", DoubleArray(vec![-5.0, 1.0, 0.0])),
        //     ("DevVarStringArray", StringArray(vec![vec![b'a', b'b'],
        //                                            vec![b'c'], vec![b'd']])),
        //     ("DevVarLongStringArray", LongStringArray(vec![-5, 1, 0, 1],
        //                                               vec![vec![b'a', b'b'], vec![b'c']])),
        //     ("DevVarDoubleStringArray", DoubleStringArray(vec![-5.0, 1.0, 0.0],
        //                                                  vec![vec![b'a', b'b'], vec![b'c']])),
        //     // no test methods for: DevEncoded, DevVarBooleanArray
        //     ];

        // for (cmd, data) in tests {
        //     println!("{}    {}", cmd, data);
        // }

        // DevVoid    <Void>
        // DevBoolean    true
        // DevShort    -147
        // DevLong    -1048576
        // DevFloat    42.42
        // DevDouble    123.456790123752
        // DevUShort    137
        // DevULong    1048576
        // DevLong64    -1152921504606846976
        // DevULong64    1152921504606846976
        // DevString    some_str_ing
        // DevVarCharArray    [1, 5, 7]
        // DevVarShortArray    [-5, 1, 0]
        // DevVarUShortArray    [5, 1, 0]
        // DevVarLongArray    [-1048576, 1, 0]
        // DevVarULongArray    [1073741824, 1, 0]
        // DevVarLong64Array    [-1152921504606846976, 1, 0]
        // DevVarULong64Array    [1152921504606846976, 1, 0]
        // DevVarFloatArray    [-42.4, 0, 80.123]
        // DevVarDoubleArray    [-5, 1, 0]
        // DevVarStringArray    [ab, c, d]
        // DevVarLongStringArray    [-5, 1, 0, 1][ab, c]
        // DevVarDoubleStringArray    [-5, 1, 0][ab, c]

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
            ("DevVarStringArray", "[ab, c, d]"),
            ("DevVarLongStringArray", "[-5, 1, 0, 1][ab, c]"),
            ("DevVarDoubleStringArray", "[-5, 1, 0][ab, c]"),
        ];

        for (cmd, data) in test_strings {
            println!("Command: {}, Value: {}", cmd, data);
            let command_data = CommandData::from_str(data);
            let res = dev.command_inout(cmd, command_data).expect(
                "Could not execute command on sys/tg_test/1, is \
                                the TangoTest server running?",
            );
            println!("\nResult: {}", res);
        }
        Ok(())
    }
}

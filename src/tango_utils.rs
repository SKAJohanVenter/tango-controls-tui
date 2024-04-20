use log::error;
use ratatui::widgets::Row;
use std::{collections::BTreeMap, error::Error};
use tango_controls_client_sys::database_proxy::DatabaseProxy;
use tango_controls_client_sys::types::{AttributeData, AttributeInfo, CmdArgType, CommandInfo};
use tui_tree_widget::TreeItem;

use crate::attribute_utils::get_attribute_list;
use crate::command_utils::get_command_list;

#[derive(Debug, Default, Clone)]
pub struct DeviceAttribute {
    pub attribute_info: AttributeInfo,
    pub attribute_data: Option<AttributeData>,
}

impl From<DeviceAttribute> for Vec<Row<'_>> {
    fn from(val: DeviceAttribute) -> Self {
        let mut rows: Vec<Row> = Vec::new();
        let data_type: CmdArgType = val.attribute_info.clone().into();
        rows.push(Row::new(["name".to_string(), val.attribute_info.name]));
        rows.push(Row::new([
            "writable".to_string(),
            val.attribute_info.writable.to_string(),
        ]));
        rows.push(Row::new(["data_type".to_string(), data_type.to_string()]));
        rows.push(Row::new([
            "data_format".to_string(),
            val.attribute_info.data_format.to_string(),
        ]));
        rows.push(Row::new([
            "max_dim_x".to_string(),
            val.attribute_info.max_dim_x.to_string(),
        ]));
        rows.push(Row::new([
            "max_dim_y".to_string(),
            val.attribute_info.max_dim_y.to_string(),
        ]));
        rows.push(Row::new([
            "description".to_string(),
            val.attribute_info.description.to_string(),
        ]));
        rows.push(Row::new([
            "label".to_string(),
            val.attribute_info.label.to_string(),
        ]));
        rows.push(Row::new([
            "unit".to_string(),
            val.attribute_info.unit.to_string(),
        ]));
        rows.push(Row::new([
            "standard_unit".to_string(),
            val.attribute_info.standard_unit.to_string(),
        ]));
        rows.push(Row::new([
            "display_unit".to_string(),
            val.attribute_info.display_unit.to_string(),
        ]));
        rows.push(Row::new([
            "format".to_string(),
            val.attribute_info.format.to_string(),
        ]));
        rows.push(Row::new([
            "min_value".to_string(),
            val.attribute_info.min_value.to_string(),
        ]));
        rows.push(Row::new([
            "max_value".to_string(),
            val.attribute_info.max_value.to_string(),
        ]));
        rows.push(Row::new([
            "min_alarm".to_string(),
            val.attribute_info.min_alarm.to_string(),
        ]));
        rows.push(Row::new([
            "max_alarm".to_string(),
            val.attribute_info.max_alarm.to_string(),
        ]));
        rows.push(Row::new([
            "writable_attr_name".to_string(),
            val.attribute_info.writable_attr_name.to_string(),
        ]));
        rows.push(Row::new([
            "disp_level".to_string(),
            format!("{}", val.attribute_info.disp_level.clone()),
        ]));
        rows
    }
}

#[derive(Debug, Default, Clone)]
pub struct Member {
    pub device_name: String,
    pub name: String,
    pub attributes: Vec<DeviceAttribute>,
    pub commands: Vec<CommandInfo>,
}

impl Member {
    fn add_attributes(&mut self) {
        if let Ok(attributes) = get_attribute_list(&self.device_name) {
            self.attributes = attributes;
        }
    }

    fn add_commands(&mut self) {
        if let Ok(commands) = get_command_list(&self.device_name) {
            self.commands = commands;
        }
    }

    pub fn attribute_to_rows(&self, attr_index: usize) -> Vec<Row> {
        let mut rows: Vec<Row> = Vec::new();

        if let Some(dev_attr) = self.attributes.get(attr_index) {
            rows.push(Row::new([
                "name".to_string(),
                dev_attr.attribute_info.name.clone(),
            ]));
        }
        rows
    }
}

#[derive(Debug, Default, Clone)]
pub struct Family {
    pub members: BTreeMap<String, Member>,
    pub name: String,
}

#[derive(Debug, Default, Clone)]
pub struct Domain {
    pub families: BTreeMap<String, Family>,
    pub name: String,
}

pub enum TreeSelection {
    Domain(Domain),
    Family(Family),
    Member(Member),
    Attribute(String, Box<DeviceAttribute>),
    Command(String, CommandInfo),
    None,
}

#[derive(Debug, Default, Clone)]
pub struct TangoDevicesLookup<'a> {
    pub domains: BTreeMap<String, Domain>,
    pub devices: Vec<String>,
    pub tree_items: Vec<TreeItem<'a, String>>,
}

pub trait GetTreeItems<'a> {
    fn get_tree_items(&self) -> Vec<TreeItem<'a, String>>;
}

impl<'a> GetTreeItems<'a> for Member {
    fn get_tree_items(&self) -> Vec<TreeItem<'a, String>> {
        let mut commands = Vec::new();
        for command in &self.commands {
            commands.push(TreeItem::new_leaf(
                command.cmd_name.clone(),
                command.cmd_name.clone(),
            ));
        }

        let mut attributes = Vec::new();
        for attribute in &self.attributes {
            attributes.push(TreeItem::new_leaf(
                attribute.attribute_info.name.clone(),
                attribute.attribute_info.name.clone(),
            ));
        }

        let items = vec![
            TreeItem::new("Attributes".to_string(), "Attributes", attributes).unwrap(),
            TreeItem::new("Commands".to_string(), "Commands", commands).unwrap(),
        ];
        items
    }
}

impl<'a> GetTreeItems<'a> for Family {
    fn get_tree_items(&self) -> Vec<TreeItem<'a, String>> {
        let mut items = Vec::new();
        for member in self.members.values() {
            items.push(
                TreeItem::new(
                    member.name.clone(),
                    member.name.clone(),
                    member.get_tree_items(),
                )
                .unwrap(),
            );
        }
        items
    }
}

impl<'a> GetTreeItems<'a> for Domain {
    fn get_tree_items(&self) -> Vec<TreeItem<'a, String>> {
        let mut items: Vec<TreeItem<'a, String>> = Vec::new();
        for (family_key, family) in &self.families {
            items.push(
                TreeItem::new(
                    family_key.clone(),
                    family_key.clone(),
                    family.get_tree_items(),
                )
                .unwrap(),
            )
        }
        items
    }
}

impl<'a> GetTreeItems<'a> for TangoDevicesLookup<'a> {
    fn get_tree_items(&self) -> Vec<TreeItem<'a, String>> {
        let mut items: Vec<TreeItem<'a, String>> = Vec::new();
        for (domain_key, domain) in &self.domains {
            items.push(
                TreeItem::new(
                    domain_key.clone(),
                    domain_key.clone(),
                    domain.get_tree_items(),
                )
                .unwrap(),
            )
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
    pub fn command_info_to_rows(command_info: CommandInfo) -> Vec<Row<'a>> {
        let rows: Vec<Row> = vec![
            Row::new(["Name".to_string(), command_info.cmd_name.clone()]),
            Row::new([
                "Display Level".to_string(),
                command_info.disp_level.to_string(),
            ]),
            Row::new(["In Type".to_string(), command_info.in_type.to_string()]),
            Row::new([
                "In Type Description".to_string(),
                command_info.in_type_desc.to_string(),
            ]),
            Row::new(["Out Type".to_string(), command_info.out_type.to_string()]),
            Row::new([
                "Out Type Description".to_string(),
                command_info.out_type_desc,
            ]),
        ];
        rows
    }

    pub fn get_tree_selection(&self, selection: Vec<String>) -> TreeSelection {
        let mut tree_selection = TreeSelection::None;
        match &selection[..] {
            [domain_ix] => {
                if let Some(domain) = self.get_domain_by_ix(domain_ix) {
                    tree_selection = TreeSelection::Domain(domain)
                }
            }
            [domain_ix, family_ix] => {
                if let Some(domain) = self.get_domain_by_ix(domain_ix) {
                    if let Some(family) = self.get_family_by_ix(&domain, family_ix) {
                        tree_selection = TreeSelection::Family(family)
                    }
                }
            }
            [domain_ix, family_ix, member_ix] => {
                if let Some(domain) = self.get_domain_by_ix(domain_ix) {
                    if let Some(family) = self.get_family_by_ix(&domain, family_ix) {
                        if let Some(member) = self.get_member_by_ix(&family, member_ix) {
                            tree_selection = TreeSelection::Member(member)
                        }
                    }
                }
            }
            [domain_ix, family_ix, member_ix, _] => {
                if let Some(domain) = self.get_domain_by_ix(domain_ix) {
                    if let Some(family) = self.get_family_by_ix(&domain, family_ix) {
                        if let Some(member) = self.get_member_by_ix(&family, member_ix) {
                            tree_selection = TreeSelection::Member(member)
                        }
                    }
                }
            }
            [domain_ix, family_ix, member_ix, attr_comm, attr_comm_ix] => {
                if let Some(domain) = self.get_domain_by_ix(domain_ix) {
                    if let Some(family) = self.get_family_by_ix(&domain, family_ix) {
                        if let Some(member) = self.get_member_by_ix(&family, member_ix) {
                            match attr_comm.as_str() {
                                "Attributes" => {
                                    if let Some(attribute_pos) = member
                                        .attributes
                                        .iter()
                                        .position(|a| a.attribute_info.name == *attr_comm_ix)
                                    {
                                        tree_selection = TreeSelection::Attribute(
                                            member.device_name,
                                            Box::new(
                                                member
                                                    .attributes
                                                    .get(attribute_pos)
                                                    .unwrap()
                                                    .clone(),
                                            ),
                                        );
                                    }
                                }
                                "Commands" => {
                                    if let Some(comm_pos) = member
                                        .commands
                                        .iter()
                                        .position(|c| c.cmd_name == *attr_comm_ix)
                                    {
                                        tree_selection = TreeSelection::Command(
                                            member.device_name,
                                            member.commands.get(comm_pos).unwrap().clone(),
                                        );
                                    }
                                }
                                _ => {
                                    error!("Should not get here")
                                }
                            }
                        }
                    }
                }
            }
            _ => (),
        }
        tree_selection
    }

    pub fn get_domain_by_ix(&self, ix: &String) -> Option<Domain> {
        self.domains.get(ix).cloned()
    }

    pub fn get_family_by_ix(&self, domain: &Domain, ix: &String) -> Option<Family> {
        domain.families.get(ix).cloned()
    }

    pub fn get_member_by_ix(&self, family: &Family, ix: &String) -> Option<Member> {
        family.members.get(ix).cloned()
    }

    pub fn get_attribute_by_ix(&self, member: &Member, attr_ix: &usize) -> Option<DeviceAttribute> {
        if let Some(attr) = member.attributes.get(*attr_ix) {
            return Some(attr.clone());
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
                    domain.name = domain_key.to_string();
                    domain
                        .families
                        .entry(family_key.to_string())
                        .or_insert_with(Family::default);

                    if let Some(family) = domain.families.get_mut(family_key) {
                        family.name = family_key.to_string();
                        let mut new_member = Member {
                            device_name: device.to_string(),
                            name: member_key.to_string(),
                            attributes: Vec::new(),
                            commands: Vec::new(),
                        };
                        new_member.add_attributes();
                        new_member.add_commands();

                        // Init the members
                        family
                            .members
                            .entry(member_key.to_string())
                            .or_insert(new_member);
                    }
                }
            }
        }
        domains
    }
}

pub fn _split_strip_string(data: &str) -> Vec<String> {
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
            assert_eq!(_split_strip_string(test_case), expected_result)
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
}

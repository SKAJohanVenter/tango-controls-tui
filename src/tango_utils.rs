use std::{collections::BTreeMap, error::Error};
use tango_client::DatabaseProxy;
use tui_tree_widget::TreeItem;

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

impl<'a> TangoDevicesLookup<'a> {
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

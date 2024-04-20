use log::error;
use std::error::Error;
use tango_controls_client_sys::device_proxy::DeviceProxy;
use tango_controls_client_sys::types::AttributeData;

use crate::tango_utils::DeviceAttribute;

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

use hidapi::HidApi;
use hidpp::features::{DeviceFriendlyName, DeviceInformation, DeviceTypeAndName, Feature};

fn main() {
    let api = HidApi::new().unwrap();
    let mut foo = 0;
    for info in api.device_list() {
        // let make = info.manufacturer_string().unwrap_or("(who?)");
        let model = info.product_string().unwrap_or("(what?)");
        println!("{model}");

        if !model.contains("POP") {
            continue;
        }
        if foo < 1 {
            foo += 1;
            continue;
        }

        let hid = info.open_device(&api).unwrap();
        let mouse = hidpp::HidppDevice::new(hid, 0xFF);

        let dev_info = DeviceInformation::open(&mouse).unwrap().unwrap();
        let info = dev_info.get_device_info().unwrap();
        println!("{info:?}");

        for i in 0..info.entity_cnt {
            let fw_info = dev_info.get_fw_info(i).unwrap();
            println!("{fw_info:?}");
        }

        let name_type = DeviceTypeAndName::open(&mouse).unwrap().unwrap();
        let len = name_type.get_device_name_count().unwrap();
        let mut buf = String::new();
        while buf.len() < len as usize {
            buf += &*name_type.get_device_name(buf.len() as u8).unwrap();
        }
        println!("{buf:?}");
        println!("{:?}", name_type.get_device_type());

        let friendly = DeviceFriendlyName::open(&mouse).unwrap().unwrap();
        let lens = friendly.get_friendly_name_len().unwrap();
        buf.clear();
        while buf.len() < lens.name_len as usize {
            buf += &*friendly.get_friendly_name(buf.len() as u8).unwrap();
        }
        println!("{buf:?}");
    }
}

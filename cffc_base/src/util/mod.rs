pub mod intr_seg_queue;
pub mod intr_queue;
pub mod multipart_form;
pub mod logger;
pub mod delay_queue;
pub mod serial_process;
pub mod utils;

pub fn get_local_ips() -> Vec<String> {
    use pnet::datalink;
    use pnet::ipnetwork::IpNetwork;

    let mut ips = Vec::new();

    for ifc in datalink::interfaces() {
        for ipn in ifc.ips {
            if let IpNetwork::V4(v) = ipn {
                let ip = v.ip();
                if !ip.is_loopback() {
                    ips.push(ip.to_string());
                }
            }
        }
    }
    ips
}
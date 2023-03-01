#[derive(Debug)]
pub struct Chain<'a> {
    pub name: &'a str,
    pub logo: &'a str,
    pub gecko: Option<&'a str>,
    pub epoch: bool,
    pub prefix: &'a str,
    pub main_denom: String,
    pub rpc_url: &'a str,
    pub rest_url: &'a str,
    pub wss_url: &'a str,
    pub decimals: u8,
    pub decimals_pow: u64,
    pub sdk_version: u8,
    pub manual_versioning: bool,
    pub json_rpc: Option<&'a str>,
}
fn main() {
    println!("bypassing build script");
}

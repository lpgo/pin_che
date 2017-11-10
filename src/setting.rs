
use std::sync::RwLock;
use config::Config;
use config::File;

lazy_static! {
	static ref SETTINGS: RwLock<Config> = get_config();
}

fn get_config() ->  RwLock<Config> {
	let mut settings = Config::default();
	settings.merge(File::with_name("config")).unwrap();
	RwLock::new(settings)
}

pub fn get_str(key: &str) -> String {
	let setting = SETTINGS.read().unwrap();
	setting.get_str(key).unwrap()
}


